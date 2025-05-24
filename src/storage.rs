use std::{
    collections::BTreeSet,
    fs::{File, create_dir_all, rename},
    path::PathBuf,
    sync::Arc,
};

use anyhow::Context;
use log::warn;
use safe_join::SafeJoin;
use serde::{Deserialize, Serialize};
use tai_time::TaiTime;

use crate::{MapEntry, fetched_data_set::FetchDataExtra};

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct StoredData {
    pub private: StoredDataPrivate,
    pub public: StoredDataPublic,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct StoredDataPublic {
    pub entries: BTreeSet<MapEntry>,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct StoredDataPrivate {
    pub last_updated: Option<TaiTime<0>>,
}

pub struct Storage {
    pub data: StoredData,
    storage_public_file: PathBuf,
    storage_private_file: PathBuf,
    extra: Arc<FetchDataExtra>,
}

impl Storage {
    pub fn new(storage_file_name: String, extra: Arc<FetchDataExtra>) -> anyhow::Result<Self> {
        let storage_public_file = extra
            .save_storage_dir
            .safe_join(&storage_file_name)
            .with_context(|| {
                format!(
                    "Error joining directory {:?} and file name {:?}",
                    extra.save_storage_dir, storage_file_name
                )
            })?;
        let storage_private_file = extra
            .save_storage_dir
            .safe_join(format!("{storage_file_name}.private"))
            .with_context(|| {
                format!(
                    "Error joining directory {:?} and file name {:?}",
                    extra.save_storage_dir, storage_file_name
                )
            })?;
        //NOTE: if that was done for security, I would need to make sure it does not override a .git files.
        //(and most likely just make sure it’s indeed a file name with not path separator)

        Ok(Self {
            data: StoredData::default(),
            storage_public_file,
            storage_private_file,
            extra,
        })
    }

    pub fn get_extra(&self) -> &Arc<FetchDataExtra> {
        &self.extra
    }

    pub fn get_storage_file(&self) -> &PathBuf {
        &self.storage_public_file
    }

    fn load_private(&mut self) -> anyhow::Result<()> {
        let mut f = File::open(&self.storage_private_file).with_context(|| {
            format!(
                "Trying to open private storage file {:?}",
                &self.storage_private_file
            )
        })?;

        self.data.private = serde_json::de::from_reader(&mut f).with_context(|| {
            format!(
                "Trying to read private storage at {:?}",
                &self.storage_private_file
            )
        })?;

        Ok(())
    }

    pub fn load(&mut self) -> anyhow::Result<()> {
        let mut f: File = File::open(&self.storage_public_file).with_context(|| {
            format!(
                "Trying to open public storage file {:?}",
                &self.storage_public_file
            )
        })?;

        self.data.public = serde_json::de::from_reader(&mut f).with_context(|| {
            format!(
                "Trying to read public storage at {:?}",
                &self.storage_public_file
            )
        })?;

        match self.load_private() {
            Ok(_) => (),
            Err(err) => {
                warn!("Failed to load private storage: {err:?}");
                self.data.private = StoredDataPrivate::default();
            }
        };
        Ok(())
    }

    pub fn save(&mut self) -> anyhow::Result<()> {
        if let Some(parent) = self.storage_public_file.parent() {
            create_dir_all(parent)
                .with_context(|| format!("Could not create dir at {parent:?}"))?;
        }

        let mut temp_path: PathBuf = self.storage_public_file.clone();
        temp_path.set_file_name(format!(
            "{}.tmp",
            self.storage_public_file
                .file_name()
                .with_context(|| format!(
                    "Can’t save storage at {:?} due to issue determining file path",
                    self.storage_public_file
                ))?
                .to_string_lossy()
        ));

        {
            let mut f_out = File::create(&temp_path)
                .with_context(|| format!("Could not create/truncate file at {:?}", &temp_path))?;
            serde_json::ser::to_writer_pretty(&mut f_out, &self.data.public)
                .with_context(|| format!("Could not write storage to {:?}", &temp_path))?;
        }
        rename(&temp_path, &self.storage_public_file)?;

        {
            let mut f_out = File::create(&temp_path)
                .with_context(|| format!("Could not create/truncate file at {:?}", &temp_path))?;
            serde_json::ser::to_writer_pretty(&mut f_out, &self.data.private)
                .with_context(|| format!("Could not write storage to {:?}", &temp_path))?;
        }
        rename(&temp_path, &self.storage_private_file)?;
        Ok(())
    }
}
