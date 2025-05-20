use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{ElementId, MapEntry};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideEntry {
    pub local_image: Option<String>,
    pub image_source_url: Option<String>,
}

impl OverrideEntry {
    pub fn override_map_entry(&self, map_entry: &mut MapEntry) {
        if let Some(local_image) = &self.local_image {
            map_entry.image = Some(format!("/images/{}", local_image.clone())); // client will escape HTML
        }

        if let Some(image_source_url) = &self.image_source_url {
            map_entry.image_source_url = Some(image_source_url.clone());
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Overrides {
    pub osm: HashMap<u64, OverrideEntry>,
    pub wikidata: HashMap<String, OverrideEntry>,
}

impl Overrides {
    pub fn get_override(&self, element_id: &ElementId) -> Option<&OverrideEntry> {
        match element_id {
            ElementId::Osm(id) => self.osm.get(id),
            ElementId::Wikidata(id) => self.wikidata.get(id),
        }
    }
}
