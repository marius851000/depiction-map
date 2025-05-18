use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::ElementId;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct MapEntry {
    pub pos: Option<(OrderedFloat<f64>, OrderedFloat<f64>)>, // WGS84
    pub name: Option<String>,
    pub location_name: Option<String>,
    pub image: Option<String>,
    pub image_source_url: Option<String>,
    pub source_url: Option<String>,
    pub is_in_exhibit: bool,
    pub nature: Option<String>,
    pub element_ids: Vec<ElementId>,
}
