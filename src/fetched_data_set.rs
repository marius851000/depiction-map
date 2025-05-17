use std::{
    collections::{BTreeSet, HashSet},
    path::PathBuf,
};

use anyhow::Context;
use log::{info, warn};
use tai_time::TaiTime;

use crate::{DepictionCategory, FetchData, MapEntry, Overrides, Storage};

pub struct FetchedDataEntry {
    pub storage: Storage,
    pub fetcher: Box<dyn FetchData + Send>,
    pub depict: BTreeSet<DepictionCategory>,
}

impl FetchedDataEntry {
    pub fn should_be_updated(&self, current_time: TaiTime<0>) -> bool {
        if let Some(fetched_time) = &self.storage.data.last_updated {
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
            self.storage.data.last_updated = Some(current_time); // Set first but not save, so it will still wait if an error occur (but will retry when restarted or just later)
            self.storage.data.entries = self
                .fetcher
                .fetch_data()
                .with_context(|| format!("Fetching data from {:?}", self.fetcher.title()))?;
            self.storage
                .save()
                .with_context(|| format!("Saving data of {:?}", self.fetcher.title()))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

pub struct FetchedDataSet {
    pub default_storage_dir: PathBuf,
    pub entries: Vec<FetchedDataEntry>,
    pub overrides: Overrides,
}

impl FetchedDataSet {
    pub fn new(default_storage_dir: PathBuf, overrides: Overrides) -> Self {
        Self {
            default_storage_dir,
            entries: Vec::new(),
            overrides,
        }
    }

    pub fn add_fetcher<T: FetchData + Send + 'static>(
        &mut self,
        fetch_data: T,
        depict: Vec<DepictionCategory>,
        storage_file_name: String,
    ) {
        let mut storage_path = self.default_storage_dir.clone();
        storage_path.push(storage_file_name);

        let mut storage = Storage::new(storage_path.clone());
        match storage.load() {
            Ok(_) => (),
            Err(err) => warn!("Failed to load some data at {:?}: {:#}", storage_path, err),
        };

        self.entries.push(FetchedDataEntry {
            storage,
            fetcher: Box::new(fetch_data),
            depict: depict.into_iter().collect(),
        });
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
                for map_entry in source_entry.storage.data.entries.iter() {
                    let mut map_entry = map_entry.clone();
                    for element_id in map_entry.element_ids.clone().iter() {
                        if let Some(override_entry) = self.overrides.get_override(element_id) {
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
