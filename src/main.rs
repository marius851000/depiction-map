use std::{
    fs::File,
    path::PathBuf,
    process::exit,
    thread::{sleep, spawn},
    time::Duration,
};

use actix_files::Files;
use actix_web::{
    App, Either, HttpResponse, HttpServer, Responder, get,
    http::{StatusCode, header::ContentType},
    rt::task::spawn_blocking,
    web::{self, Data},
};
use clap::Parser;
use depiction_map::{
    DepictAppData, DepictionCategory, FetchDataOpenStreetMap, FetchDataWikidataSparql,
    FetchedDataSet, Overrides,
};
use env_logger::Env;
use log::{error, info};
use mime_guess::from_path;
use rust_embed::Embed;

// based on https://git.sr.ht/~pyrossh/rust-embed/tree/master/item/examples/actix.rs (for the static file delivery)
#[derive(Embed)]
#[folder = "static"]
struct Asset;

fn handle_embedded_file(path: &str) -> HttpResponse {
    match Asset::get(path) {
        Some(content) => HttpResponse::Ok()
            .content_type(from_path(path).first_or_octet_stream().as_ref())
            .body(content.data.into_owned()),
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}

#[actix_web::get("/")]
async fn index() -> impl Responder {
    handle_embedded_file("index.html")
}

#[actix_web::get("/static/{_:.*}")]
async fn static_ressources(path: web::Path<String>) -> impl Responder {
    handle_embedded_file(path.as_str())
}

#[get("/depiction/{category}.json")]
async fn get_depiction(
    category: web::Path<String>,
    data: Data<DepictAppData>,
) -> Either<HttpResponse, (&'static str, StatusCode)> {
    let category = DepictionCategory(category.into_inner());
    match data.display_data_set.to_display.get(&category) {
        Some(display_entry) => Either::Left(
            HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(display_entry.load().json.clone()),
        ),
        None => Either::Right(("category does not exist", StatusCode::NOT_FOUND)),
    }
}

#[derive(Parser, Debug)]
pub struct Opts {
    ressource_path: PathBuf,
    save_path: PathBuf,
    #[arg(long, default_value = "8080")]
    port: u16,
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
}

#[actix_web::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let opts = Opts::parse();

    let app_data = spawn_blocking(move || {
        let overrides_path = opts.ressource_path.join("overrides.json");
        let overrides_file = File::open(&overrides_path).unwrap();
        let overrides: Overrides = serde_json::from_reader(overrides_file).unwrap();

        let mut fetched_data_set = FetchedDataSet::new(opts.save_path, overrides).unwrap();

        let osm_dragon_fetcher = FetchDataOpenStreetMap {
            api: FetchDataOpenStreetMap::default_api(),
            query: "[out:json][timeout:30];

            nwr[\"artwork_subject\"~\"dragon\"][\"artwork_subject\"!~\"dragonfl\"]; // but what about both depiction of dragon and dragonfly? Does not appear to exist for now, but that really show that OSM data model is innapropriate for that kind of use
            // idea: just get all dragon and then post-process locally

            out geom;".to_string(),
            title: "Dragons from OpenStreetMap".to_string(),
        };

        fetched_data_set.add_fetcher(
            osm_dragon_fetcher,
            vec![DepictionCategory::dragon()],
            "osm_dragon.json".into(),
        ).unwrap();

        let wikidata_dragon_fetcher = FetchDataWikidataSparql::new(include_str!("../wikidata_dragon_query.sparql").to_string(), "dragon from wikidata".into()).unwrap();
        fetched_data_set.add_fetcher(
            wikidata_dragon_fetcher,
            vec![DepictionCategory::dragon()],
            "wikidata_dragon.json".into(),
        ).unwrap();

        let mut app_data = DepictAppData::new(&fetched_data_set, opts.ressource_path.clone()).unwrap();
        let handle = app_data.start_update_thread(fetched_data_set);
        spawn(move || loop {
            sleep(Duration::from_secs(2));
            if handle.is_finished() {
                error!("Background update thread finished while it should never stop. Exiting.");
                exit(200);
            }
        });
        Data::new(app_data)
    }).await.unwrap();

    info!("Starting server on {}:{}", opts.host, opts.port);

    HttpServer::new(move || {
        let images_path = app_data.ressource_path.join("images");
        App::new()
            .app_data(app_data.clone())
            .service(get_depiction)
            .service(static_ressources)
            .service(index)
            .service(Files::new("/images", images_path).show_files_listing())
    })
    .bind((opts.host, opts.port))
    .unwrap()
    .run()
    .await
    .unwrap();
}
