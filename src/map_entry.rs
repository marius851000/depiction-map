use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::ImageReference;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct MapEntry {
    pub pos: (OrderedFloat<f64>, OrderedFloat<f64>), // WGS84
    pub name: Option<String>,
    pub location_name: Option<String>,
    pub distant_image: Option<ImageReference>, // Local image will overwrite this
    pub source_url: Option<String>,
    pub is_in_exhibit: bool,
}
