use std::{collections::BTreeSet, time::Duration};

use crate::MapEntry;

/**
 * A trait that describe how to fetch information about some depiction from a source
 */
pub trait FetchData {
    fn fetch_data(&self) -> anyhow::Result<BTreeSet<MapEntry>>;

    fn title(&self) -> String;

    fn retry_every(&self) -> Duration;
}
