use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{ElementId, MapEntry, MapEntryImageSource};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideEntry {
    pub local_image: Option<String>,
    pub image_source_url: Option<String>,
    pub image_source_text: Option<String>,
}

impl OverrideEntry {
    pub fn override_map_entry(&self, map_entry: &mut MapEntry) {
        if let Some(local_image) = &self.local_image {
            map_entry.image = Some(MapEntryImageSource {
                url: format!("/images/{}", local_image.clone()),
                credit_url: None,
                credit_text: None,
            })
        }

        if let Some(image) = &mut map_entry.image {
            if let Some(image_source_url) = &self.image_source_url {
                image.credit_url = Some(image_source_url.clone());
            }

            if let Some(image_source_text) = &self.image_source_text {
                image.credit_text = Some(image_source_text.clone());
            }
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
