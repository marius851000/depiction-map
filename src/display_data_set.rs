use std::collections::{HashMap, HashSet};

use arc_swap::ArcSwap;

use crate::{DepictionCategory, MapEntry};

pub struct DisplayDataSet {
    pub to_display: HashMap<DepictionCategory, ArcSwap<Vec<MapEntry>>>,
}

impl DisplayDataSet {
    pub fn new(depictions: &HashSet<&DepictionCategory>) -> Self {
        let mut to_display = HashMap::new();
        for depiction in depictions.iter() {
            to_display.insert((*depiction).clone(), ArcSwap::from_pointee(Vec::new()));
        }
        Self { to_display }
    }
}
