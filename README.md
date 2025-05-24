# Depiction Map

Finds and serves a web map depicting various things (mostly dragons) using online sources. It automates searching for geospatial data and visualizing it on a web map.

## Features

- Fetches data from both OpenStreetMap and Wikidata
- Persists data in a Git repository, allowing you to monitor changes
- Serves a basic OSM web map

## Usage

It is still in early implementation. For now, you can run the main executable (`cargo run -- ./sample_ressources ./save_folder`) and it will serve the interface on port 8080. The configuration of what to fetch is hardcoded for now (in `main.rs`).

I will probably release the configuration I use for dragons, which overrides some values on the fetched data, but contains (non-free, unlicensed) photos of those, hence why I don’t share it here.
