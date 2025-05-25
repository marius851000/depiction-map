use log::error;
use std::{
    path::PathBuf,
    sync::Arc,
    thread::{self, JoinHandle, sleep},
    time::Duration,
};

use anyhow::Context;
use arc_swap::ArcSwap;
use log::{info, warn};
use tai_time::TaiTime;

use crate::{DisplayDataSet, DisplayDataSetEntry, FetchedDataSet};

pub struct DepictAppData {
    pub display_data_set: Arc<DisplayDataSet>,
    pub ressource_path: PathBuf,
}

impl DepictAppData {
    pub fn new(fetched_data_set: &FetchedDataSet, ressource_path: PathBuf) -> anyhow::Result<Self> {
        let mut display_data_set =
            DisplayDataSet::new(&fetched_data_set.list_all_depiction_category());

        for depiction in fetched_data_set.list_all_depiction_category().clone() {
            let entries = fetched_data_set.build_data_for_depiction_category(depiction.clone());
            display_data_set.to_display.insert(
                depiction.clone(),
                ArcSwap::from_pointee(
                    DisplayDataSetEntry::new(entries)
                        .context("Storing the result in DisplayDataSetEntry")?,
                ),
            );
        }

        Ok(Self {
            display_data_set: Arc::new(display_data_set),
            ressource_path,
        })
    }

    /// Will panic if called more than once
    pub fn start_update_thread(&mut self, mut fetched_data_set: FetchedDataSet) -> JoinHandle<()> {
        let display_data_set = self.display_data_set.clone();

        thread::spawn(move || {
            info!("Update thread spawned");
            loop {
                for entry_pos in 0..fetched_data_set.entries.len() {
                    let current_time = match TaiTime::try_now() {
                        Ok(t) => t,
                        Err(err) => {
                            panic!(
                                "Something seriously wrong happened getting the current (TAI) time ({err:?}). The update thread will be destroyed."
                            )
                        }
                    };
                    match fetched_data_set.entries[entry_pos].perform_update_if_needed(current_time)
                    {
                        Ok(false) => (),
                        Ok(true) => {
                            for depiction in &fetched_data_set.entries[entry_pos].depict {
                                let map_entries = fetched_data_set
                                    .build_data_for_depiction_category(depiction.clone());
                                let display_entry =
                                    display_data_set.to_display.get(depiction).unwrap(); // All possible value should be set in the constructor (althought a bad implementation could lead to a panic here)
                                match DisplayDataSetEntry::new(map_entries) {
                                    Ok(entry) => display_entry.swap(Arc::new(entry)),
                                    Err(err) => {
                                        error!(
                                            "Failed to create the struct used to share the data with the other threads: {err:#}"
                                        );
                                        continue;
                                    }
                                };
                                info!(
                                    "Update successfull for {:?}",
                                    fetched_data_set.entries[entry_pos].fetcher.title()
                                );
                            }
                        }
                        Err(err) => {
                            warn!(
                                "Could not perform update of {:?}: {:?}",
                                fetched_data_set.entries[entry_pos].fetcher.title(),
                                err
                            )
                        }
                    }
                }
                sleep(Duration::from_secs(10));
            }
        })
    }
}
