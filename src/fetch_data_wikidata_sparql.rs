use std::{collections::BTreeSet, time::Duration};

use anyhow::{Context, bail};
use ordered_float::OrderedFloat;
use reqwest::blocking::Client;
use serde::Deserialize;
use url::Url;

use crate::{ElementId, FetchData, MapEntry, USER_AGENT};

fn parse_point(value: &str) -> Option<(f64, f64)> {
    let second_part = value.split("Point(").nth(1)?;
    let isolated_values = second_part.split(")").next()?;
    let mut space_splited = isolated_values.split(" ");
    let first_float = space_splited.next()?;
    let second_float = space_splited.next()?;
    let first_float_parsed = first_float.parse().ok()?;
    let second_float_parsed = second_float.parse().ok()?;

    Some((second_float_parsed, first_float_parsed)) // That’s an unusual ordering...
}

#[derive(Deserialize)]
struct WikidataValue {
    value: Option<String>,
}

impl WikidataValue {
    #[allow(dead_code)]
    fn is_false(&self) -> bool {
        self.value
            .as_ref()
            .map(|x| x.to_lowercase() == "false")
            .unwrap_or(false)
    }

    fn is_true(&self) -> bool {
        self.value
            .as_ref()
            .map(|x| x.to_lowercase() == "true")
            .unwrap_or(false)
    }
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct WikidataElement {
    item: Option<WikidataValue>,
    itemLabel: Option<WikidataValue>,
    coords: Option<WikidataValue>,
    coordsApproxP1_0: Option<WikidataValue>,
    coordsApproxP1_1: Option<WikidataValue>,
    coordsApproxP2_0: Option<WikidataValue>,
    coordsApproxC1_0_0: Option<WikidataValue>,
    coordsApproxC1_0_1: Option<WikidataValue>,
    image: Option<WikidataValue>,
    placeLabel: Option<WikidataValue>,
    natureLabel: Option<WikidataValue>,
    isInExhibit: Option<WikidataValue>,
}

impl WikidataElement {
    fn get_coord_value(&self) -> Option<&str> {
        let orders = [
            &self.coords,
            &self.coordsApproxP1_0,
            &self.coordsApproxP2_0,
            &self.coordsApproxP1_1,
            &self.coordsApproxC1_0_0,
            &self.coordsApproxC1_0_1,
        ];

        for element in orders.into_iter().flatten() {
            if let Some(coord) = &element.value {
                return Some(coord);
            }
        }
        None
    }

    fn is_direct_location(&self) -> bool {
        self.coords
            .as_ref()
            .map(|x| x.value.is_some())
            .unwrap_or(false)
    }
}

#[derive(Deserialize)]
struct WikidataDocumentResult {
    bindings: Vec<WikidataElement>,
}

#[derive(Deserialize)]
struct WikidataDocument {
    results: WikidataDocumentResult,
}

impl WikidataDocument {
    fn get_elements(&self) -> &Vec<WikidataElement> {
        &self.results.bindings
    }
}

pub struct FetchDataWikidataSparql {
    query: String,
    title: String,
}

impl FetchDataWikidataSparql {
    pub fn new(query: String, title: String) -> anyhow::Result<Self> {
        Ok(Self { query, title })
    }
}

impl FetchData for FetchDataWikidataSparql {
    fn fetch_data(&self) -> anyhow::Result<BTreeSet<MapEntry>> {
        let mut url_to_query = Url::parse("https://query.wikidata.org/sparql")?;

        url_to_query
            .query_pairs_mut()
            .append_pair("query", &self.query);

        let client = Client::builder().user_agent(USER_AGENT).build()?;
        let response = client
            .get(url_to_query.clone())
            .header("Accept", "application/sparql-results+json")
            .send()
            .with_context(|| format!("Performing the wikidata get query to {url_to_query}"))?;
        if !response.status().is_success() {
            bail!(
                "Wikidata request failed with status code {}, the response being {} and the url being {}",
                response.status(),
                response.text().unwrap_or("<invalid unicode>".to_string()),
                url_to_query
            );
        }

        let text = response
            .text()
            .with_context(|| format!("Could not decode encoding of {url_to_query}"))?;
        let parsed: WikidataDocument = serde_json::de::from_str(&text)
            .with_context(|| format!("Could not parse answer from {url_to_query}"))?;
        let elements = parsed.get_elements();

        let mut results = BTreeSet::new();
        for element in elements {
            let coord = element
                .get_coord_value()
                .and_then(parse_point)
                .map(|(x, y)| (OrderedFloat::from(x), OrderedFloat::from(y)));

            let item_url = match element.item.as_ref().and_then(|x| x.value.clone()) {
                Some(value) => value,
                None => bail!("Item URL missing in an entry (the query likely has an issue)"),
            };
            let qid = match item_url.split("/").last() {
                Some(value) => value.to_string(),
                None => bail!("Could not extra the qid from a (most-likely empty) wikidata URL"),
            };
            results.insert(MapEntry {
                pos: coord,
                name: element.itemLabel.as_ref().and_then(|x| x.value.clone()),
                location_name: element.placeLabel.as_ref().and_then(|x| x.value.clone()),
                image: element.image.as_ref().and_then(|x| x.value.clone()),
                image_source_url: element
                    .image
                    .as_ref()
                    .and_then(|x| x.value.as_ref())
                    .and_then(|x| Url::parse(x).ok())
                    .and_then(|url| {
                        url.path_segments()
                            .and_then(|x| x.last())
                            .map(|x| x.to_string())
                    })
                    .map(|file_url_name| {
                        format!("https://commons.wikimedia.org/wiki/File:{file_url_name}")
                    }),
                image_source_text: Some("from Wikimedia Commons".to_string()),
                source_url: Some(item_url),
                is_in_exhibit: element
                    .isInExhibit
                    .as_ref()
                    .map(|x| x.is_true())
                    .unwrap_or(false)
                    || !element.is_direct_location(),
                nature: element.natureLabel.as_ref().and_then(|x| x.value.clone()),
                element_ids: vec![ElementId::Wikidata(qid)],
            });
        }

        Ok(results)
    }

    fn retry_every(&self) -> std::time::Duration {
        Duration::from_secs(3600 * 3)
    }

    fn title(&self) -> String {
        self.title.clone()
    }
}
