use std::{
    sync::Arc,
    thread::{self, sleep},
    time::Duration,
};

use arc_swap::ArcSwap;
use log::{info, warn};
use tai_time::TaiTime;

use crate::{DisplayDataSet, FetchedDataSet};

pub struct DepictAppData {
    pub display_data_set: Arc<DisplayDataSet>,
}

impl DepictAppData {
    pub fn new(fetched_data_set: &FetchedDataSet) -> Self {
        let mut display_data_set =
            DisplayDataSet::new(&fetched_data_set.list_all_depiction_category());

        for depiction in fetched_data_set.list_all_depiction_category().clone() {
            let built = fetched_data_set.build_data_for_depiction_category(depiction.clone());
            display_data_set
                .to_display
                .insert(depiction.clone(), ArcSwap::from_pointee(built));
        }

        Self {
            display_data_set: Arc::new(display_data_set),
        }
    }

    /// Will panic if called more than once
    pub fn start_update_thread(&mut self, mut fetched_data_set: FetchedDataSet) {
        let display_data_set = self.display_data_set.clone();
        //TODO: monitor thread
        thread::spawn(move || {
            info!("Update thread spawned");
            loop {
                for entry_pos in 0..fetched_data_set.entries.len() {
                    let current_time = match TaiTime::try_now() {
                        Ok(t) => t,
                        Err(err) => {
                            panic!(
                                "Something seriously wrong happened getting the current (TAI) time ({:#}). The update thread will be destroyed.",
                                err
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
                                display_entry.swap(Arc::new(map_entries));
                                info!(
                                    "Update successfull for {:?}",
                                    fetched_data_set.entries[entry_pos].fetcher.title()
                                );
                            }
                        }
                        Err(err) => {
                            warn!(
                                "Could not perform update of {:?}: {:#}",
                                fetched_data_set.entries[entry_pos].fetcher.title(),
                                err
                            )
                        }
                    }
                }
                sleep(Duration::from_secs(10));
            }
        });
    }
}
