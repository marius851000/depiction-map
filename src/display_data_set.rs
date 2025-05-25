use std::collections::{HashMap, HashSet};

use arc_swap::ArcSwap;

use crate::{DepictionCategory, MapEntry};

pub struct DisplayDataSetEntry {
    pub entries: Vec<MapEntry>,
    pub json: String,
}

impl DisplayDataSetEntry {
    pub fn new(entries: Vec<MapEntry>) -> anyhow::Result<Self> {
        let json_string = serde_json::to_string(&entries)?;
        Ok(Self {
            entries,
            json: json_string,
        })
    }
}

impl Default for DisplayDataSetEntry {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            json: "[]".to_string(),
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
