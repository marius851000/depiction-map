use std::{
    collections::{BTreeSet, HashSet},
    fs::File,
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::{Context, bail};
use git2::{Repository, RepositoryInitOptions};
use log::{info, warn};
use pathdiff::diff_paths;
use tai_time::TaiTime;

use crate::{DepictionCategory, FetchData, MapEntry, Overrides, Storage, make_commit};

pub struct FetchedDataEntry {
    pub storage: Storage,
    pub fetcher: Box<dyn FetchData + Send>,
    pub depict: BTreeSet<DepictionCategory>,
}

impl FetchedDataEntry {
    pub fn should_be_updated(&self, current_time: TaiTime<0>) -> bool {
        if let Some(fetched_time) = &self.storage.data.private.last_updated {
            // if the clock goes backward for some reason, refetch the data (and so re-set the time)
            if current_time < *fetched_time {
                return true;
            }
            if *fetched_time + self.fetcher.retry_every() < current_time {
                return true;
            }
            false
        } else {
            true
        }
    }

    /// Return true if successfully updated, false if not needed
    pub fn perform_update_if_needed(&mut self, current_time: TaiTime<0>) -> anyhow::Result<bool> {
        if self.should_be_updated(current_time) {
            info!("Updating {:?}", self.fetcher.title());
            self.storage.data.private.last_updated = Some(current_time); // Set first but not save, so it will still wait if an error occur (but will retry when restarted or just later)
            self.storage.data.public.entries = self
                .fetcher
                .fetch_data()
                .with_context(|| format!("Fetching data from {:?}", self.fetcher.title()))?;
            self.storage
                .save()
                .with_context(|| format!("Saving data of {:?}", self.fetcher.title()))?;

            let extra = self.storage.get_extra();
            let repo = match extra.repo.lock() {
                Ok(r) => r,
                Err(err) => bail!("Failed to get repo: {:?}", err), // This error can’t be used by anyhow directly
            };

            make_commit(
                &repo,
                &diff_paths(self.storage.get_storage_file(), &extra.save_storage_dir)
                    .context("Could not diff paths for indexing with git")?,
                &format!("Update {}", self.fetcher.title()),
            )
            .context("Commiting changes to git")?;

            Ok(true)
        } else {
            Ok(false)
        }
    }
}

pub struct FetchedDataSet {
    pub entries: Vec<FetchedDataEntry>,
    pub extra: Arc<FetchDataExtra>,
}

pub struct FetchDataExtra {
    pub save_storage_dir: PathBuf,
    pub overrides: Overrides,
    pub repo: Mutex<Repository>,
}

impl FetchedDataSet {
    pub fn new(default_storage_dir: PathBuf, overrides: Overrides) -> anyhow::Result<Self> {
        let repo = match Repository::open(&default_storage_dir) {
            Ok(repo) => repo,
            Err(err) => {
                if err.code() == git2::ErrorCode::NotFound {
                    info!("Creating new storage repo in {default_storage_dir:?}");
                    Repository::init_opts(
                        &default_storage_dir,
                        RepositoryInitOptions::new().no_reinit(true),
                    )
                    .with_context(|| {
                        format!("Creating new storage repo in {default_storage_dir:?}")
                    })?
                } else {
                    return Err(Into::<anyhow::Error>::into(err)).with_context(|| {
                        format!("Opening storage repo in {default_storage_dir:?}")
                    });
                }
            }
        };

        let gitignore_path = default_storage_dir.join(".gitignore");
        if !gitignore_path.exists() {
            let mut f = File::create(&gitignore_path)
                .with_context(|| format!("Creating .gitignore in {default_storage_dir:?}"))?;
            writeln!(f, "*.private\n")
                .with_context(|| format!("Writing .gitignore in {default_storage_dir:?}"))?;
            make_commit(&repo, &PathBuf::from(".gitignore"), "Add .gitignore")?;
        }

        Ok(Self {
            entries: Vec::new(),
            extra: Arc::new(FetchDataExtra {
                save_storage_dir: default_storage_dir,
                overrides,
                repo: Mutex::new(repo),
            }),
        })
    }

    pub fn add_fetcher<T: FetchData + Send + 'static>(
        &mut self,
        fetch_data: T,
        depict: Vec<DepictionCategory>,
        storage_file_name: String,
    ) -> anyhow::Result<()> {
        let mut storage = Storage::new(storage_file_name.clone(), self.extra.clone())?;
        match storage.load() {
            Ok(_) => (),
            Err(err) => warn!(
                "Failed to load some storage at {storage_file_name}: {err:?}"
            ),
        };

        self.entries.push(FetchedDataEntry {
            storage,
            fetcher: Box::new(fetch_data),
            depict: depict.into_iter().collect(),
        });

        Ok(())
    }

    pub fn build_data_for_depiction_category(
        &self,
        depict_category: DepictionCategory,
    ) -> Vec<MapEntry> {
        //TODO: deduplication
        let mut result = Vec::new();

        for source_entry in &self.entries {
            let should_be_used = source_entry.depict.iter().any(|e| *e == depict_category);
            if should_be_used {
                for map_entry in source_entry.storage.data.public.entries.iter() {
                    let mut map_entry = map_entry.clone();
                    for element_id in map_entry.element_ids.clone().iter() {
                        if let Some(override_entry) = self.extra.overrides.get_override(element_id)
                        {
                            override_entry.override_map_entry(&mut map_entry);
                        }
                    }
                    result.push(map_entry);
                }
            }
        }

        result
    }

    pub fn list_all_depiction_category(&self) -> HashSet<&DepictionCategory> {
        let mut result = HashSet::new();
        for source_entry in &self.entries {
            for depict_category in &source_entry.depict {
                result.insert(depict_category);
            }
        }
        result
    }
}
