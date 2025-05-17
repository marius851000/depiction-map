use std::path::PathBuf;

use actix_files::Files;
use actix_web::{
    App, Either, HttpServer, get,
    http::StatusCode,
    rt::task::spawn_blocking,
    web::{self, Data, Json},
};
use clap::Parser;
use depiction_map::{
    DepictAppData, DepictionCategory, FetchDataOpenStreetMap, FetchedDataSet, MapEntry,
};
use log::info;

#[get("/depiction/{category}.json")]
async fn get_depiction(
    category: web::Path<String>,
    data: Data<DepictAppData>,
) -> Either<Json<Vec<MapEntry>>, (&'static str, StatusCode)> {
    let category = DepictionCategory(category.into_inner());
    let result = match data.display_data_set.to_display.get(&category) {
        Some(value) => value,
        None => return Either::Right(("category does not exist", StatusCode::NOT_FOUND)),
    };
    return Either::Left(web::Json(result.load().as_ref().clone()));
}

#[derive(Parser, Debug)]
pub struct Opts {
    ressource_path: PathBuf,
    #[arg(default_value = "8080")]
    port: u16,
    #[arg(default_value = "127.0.0.1")]
    host: String,
}

#[actix_web::main]
async fn main() {
    env_logger::init();

    let opts = Opts::parse();

    let app_data = spawn_blocking(move || {
        let mut fetched_data_set = FetchedDataSet::new("./test".into());

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
        );

        let mut app_data = DepictAppData::new(&fetched_data_set, opts.ressource_path.clone());
        app_data.start_update_thread(fetched_data_set);
        Data::new(app_data)
    }).await.unwrap();

    info!("Starting server on {}:{}", opts.host, opts.port);

    HttpServer::new(move || {
        let mut static_path = app_data.ressource_path.clone();
        static_path.push("static");
        App::new()
            .app_data(app_data.clone())
            .service(get_depiction)
            .service(Files::new("/", static_path).index_file("index.html"))
    })
    .bind((opts.host, opts.port))
    .unwrap()
    .run()
    .await
    .unwrap();
}
