use std::collections::{HashMap, HashSet};

use actix_web::web::Bytes;
use arc_swap::ArcSwap;

use crate::{DepictionCategory, MapEntry};

pub struct DisplayDataSetEntry {
    pub entries: Vec<MapEntry>,
    pub json: Bytes,
}

impl DisplayDataSetEntry {
    pub fn new(entries: Vec<MapEntry>) -> anyhow::Result<Self> {
        let json_string = serde_json::to_string(&entries)?;
        let json_bytes = Bytes::from(json_string);
        Ok(Self {
            entries,
            json: json_bytes,
        })
    }
}

impl Default for DisplayDataSetEntry {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            json: Bytes::from_static(b"[]"),
        }
    }
}

pub struct DisplayDataSet {
    pub to_display: HashMap<DepictionCategory, ArcSwap<DisplayDataSetEntry>>,
}

impl DisplayDataSet {
    pub fn new(depictions: &HashSet<&DepictionCategory>) -> Self {
        let mut to_display = HashMap::new();
        for depiction in depictions.iter() {
            to_display.insert((*depiction).clone(), ArcSwap::default());
        }
        Self { to_display }
    }
}
