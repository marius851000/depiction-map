use log::warn;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::ElementId;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct MapEntryImageSource {
    pub url: String,
    pub credit_url: Option<String>,
    pub credit_text: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct MapEntry {
    pub pos: Option<(OrderedFloat<f64>, OrderedFloat<f64>)>, // WGS84
    pub name: Option<String>,
    pub location_name: Option<String>,
    pub image: Option<MapEntryImageSource>,
    pub source_url: Option<String>,
    pub is_in_exhibit: bool,
    pub nature: Option<String>,
    pub element_ids: Vec<ElementId>,
}

impl MapEntry {
    /// Transform some value once this map entry is at its otherwise definitive state.
    pub fn post_process(&mut self) {
        if let Some(image) = &mut self.image {
            if image.credit_text.is_none() {
                if let Some(credit_url) = &image.credit_url {
                    match Url::parse(credit_url) {
                        Ok(url) => {
                            if let Some(domain) = url.domain() {
                                image.credit_text = Some(format!("From {}", domain));
                            } else {
                                warn!("Failed to get the url of an image source \"{}\"", image.url)
                            }
                        }
                        Err(err) => warn!(
                            "Error parsing the url of an image source \"{}\": {:#}",
                            image.url, err
                        ),
                    }
                }
            }
        }
    }
}
