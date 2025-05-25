use log::warn;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::ElementId;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct MapEntry {
    pub pos: Option<(OrderedFloat<f64>, OrderedFloat<f64>)>, // WGS84
    pub name: Option<String>,
    pub location_name: Option<String>,
    pub image: Option<String>,
    pub image_source_url: Option<String>,
    pub image_source_text: Option<String>,
    pub source_url: Option<String>,
    pub is_in_exhibit: bool,
    pub nature: Option<String>,
    pub element_ids: Vec<ElementId>,
}

impl MapEntry {
    /// Transform some value once this map entry is at its otherwise definitive state.
    pub fn post_process(&mut self) {
        if self.image_source_text.is_none() {
            if let Some(source_url) = self.image_source_url.as_ref() {
                match Url::parse(&source_url) {
                    Ok(url) => {
                        if let Some(domain) = url.domain() {
                            self.image_source_text = Some(format!("From {}", domain));
                        } else {
                            warn!(
                                "Failed to get the url of an image source \"{}\"",
                                source_url
                            )
                        }
                    }
                    Err(err) => warn!(
                        "Error parsing the url of an image source \"{}\": {:#}",
                        source_url, err
                    ),
                }
            }
        }
    }
}
