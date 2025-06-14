use std::{collections::BTreeSet, time::Duration};

use anyhow::bail;
use log::warn;
use osm_overpass::api::{NWR, OverpassAPI};

use crate::{ElementId, FetchData, MapEntry};

#[allow(clippy::single_match)]
fn guess_nature_from_tags(tags: &std::collections::HashMap<String, String>) -> Option<String> {
    for (k, v) in tags {
        match (k.as_str(), v.as_str()) {
            ("artwork_type", v) => return Some(v.to_string()),
            _ => (),
        }
    }
    None
}

pub struct FetchDataOpenStreetMap {
    pub query: String,
    pub api: OverpassAPI,
    pub title: String,
}

impl FetchDataOpenStreetMap {
    pub fn default_api() -> OverpassAPI {
        OverpassAPI::new("https://overpass-api.de/api/interpreter".to_string())
    }
}
impl FetchData for FetchDataOpenStreetMap {
    fn fetch_data(&self) -> anyhow::Result<BTreeSet<MapEntry>> {
        let overpass_result = match self.api.query_sync(&self.query) {
            Ok(x) => x,
            Err(err) => bail!(err),
        };

        Ok(overpass_result
            .filter_map(|element| {
                let tag;
                let center;
                let osm_id;
                match element {
                    NWR::Node(node) => {
                        tag = node.tags.clone();
                        center = node.location;
                        osm_id = node.osm_id;
                    }
                    NWR::Way(way) => {
                        warn!("Ignored OSM way {}", way.osm_id);
                        return None;
                    }
                    NWR::Relation(rel) => {
                        warn!("Ignored OSM relation {}", rel.osm_id);
                        return None;
                    }
                };
                Some(MapEntry {
                    pos: Some((center.0.into(), center.1.into())),
                    image: None,
                    location_name: None,
                    name: tag.get("name").map(|x| x.to_string()),
                    source_url: Some(format!("https://www.openstreetmap.org/node/{osm_id}")),
                    source_text: "From OpenStreetMap".into(),
                    is_in_exhibit: false,
                    nature: guess_nature_from_tags(&tag),
                    element_ids: vec![ElementId::Osm(osm_id)],
                })
            })
            .collect())
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn retry_every(&self) -> Duration {
        Duration::from_secs(3600 * 3)
    }
}
