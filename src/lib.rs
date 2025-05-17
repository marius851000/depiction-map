mod map_entry;
pub use map_entry::MapEntry;

mod image_reference;
pub use image_reference::ImageReference;

mod storage;
use serde::{Deserialize, Serialize};
pub use storage::Storage;

mod fetch_data;
pub use fetch_data::FetchData;

mod fetch_data_openstreetmap;
pub use fetch_data_openstreetmap::FetchDataOpenStreetMap;

mod fetched_data_set;
pub use fetched_data_set::FetchedDataSet;

mod display_data_set;
pub use display_data_set::DisplayDataSet;

mod depict_app_data;
pub use depict_app_data::DepictAppData;

/**
 * What kind of stuff depict this. Allow to group multiple source together.
 */
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct DepictionCategory(pub String);

impl DepictionCategory {
    pub fn dragon() -> Self {
        Self("dragon".into())
    }
}
