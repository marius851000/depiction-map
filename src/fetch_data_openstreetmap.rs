use std::{collections::BTreeSet, time::Duration};

use anyhow::bail;
use log::warn;
use osm_overpass::api::{NWR, OverpassAPI};

use crate::{FetchData, MapEntry};

pub struct FetchDataOpenStreetMap {
    pub query: String,
    pub api: OverpassAPI,
    pub title: String,
}

/*
[out:json][timeout:30];

nwr["artwork_subject"~"dragon"]["artwork_subject"!~"dragonfl"]; // but what about both depiction of dragon and dragonfly? Does not appear to exist for now, but that really show that OSM data model is innapropriate for that matter

out geom;*/

impl FetchDataOpenStreetMap {
    pub fn default_api() -> OverpassAPI {
        return OverpassAPI::new("https://overpass-api.de/api/interpreter".to_string());
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
                    pos: (center.0.into(), center.1.into()),
                    distant_image: None,
                    location_name: None,
                    name: tag.get("name").map(|x| x.to_string()),
                    source_url: Some(format!("https://www.openstreetmap.org/node/{}", osm_id)),
                    is_in_exhibit: false,
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
