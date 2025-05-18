use std::{
    collections::BTreeSet,
    fs::{File, create_dir_all, rename},
    path::PathBuf,
    sync::Arc,
};

use anyhow::Context;
use safe_join::SafeJoin;
use serde::{Deserialize, Serialize};
use tai_time::TaiTime;

use crate::{MapEntry, fetched_data_set::FetchDataExtra};

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct StoredData {
    pub last_updated: Option<TaiTime<0>>,
    pub entries: BTreeSet<MapEntry>,
}

pub struct Storage {
    pub data: StoredData,
    storage_file: PathBuf,
    extra: Arc<FetchDataExtra>,
}

impl Storage {
    pub fn new(storage_file_name: String, extra: Arc<FetchDataExtra>) -> anyhow::Result<Self> {
        let storage_file = extra
            .save_storage_dir
            .safe_join(&storage_file_name)
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
            storage_file,
            extra,
        })
    }

    pub fn get_extra(&self) -> &Arc<FetchDataExtra> {
        &self.extra
    }

    pub fn get_storage_file(&self) -> &PathBuf {
        &self.storage_file
    }

    pub fn load(&mut self) -> anyhow::Result<()> {
        let mut f: File = File::open(&self.storage_file)
            .with_context(|| format!("Trying to open storage file {:?}", &self.storage_file))?;

        self.data = serde_json::de::from_reader(&mut f)
            .with_context(|| format!("Trying to read storage at {:?}", &self.storage_file))?;
        Ok(())
    }

    pub fn save(&mut self) -> anyhow::Result<()> {
        if let Some(parent) = self.storage_file.parent() {
            create_dir_all(parent)
                .with_context(|| format!("Could not create dir at {:?}", parent))?;
        }

        let mut temp_path: PathBuf = self.storage_file.clone();
        temp_path.set_file_name(format!(
            "{}.tmp",
            self.storage_file
                .file_name()
                .with_context(|| format!(
                    "Can’t save storage at {:?} due to issue determining file path",
                    self.storage_file
                ))?
                .to_string_lossy()
        ));

        {
            let mut f_out = File::create(&temp_path)
                .with_context(|| format!("Could not create/truncate file at {:?}", &temp_path))?;
            serde_json::ser::to_writer_pretty(&mut f_out, &self.data)
                .with_context(|| format!("Could not write storage to {:?}", &temp_path))?;
        }

        rename(&temp_path, &self.storage_file)?;
        Ok(())
    }
}
