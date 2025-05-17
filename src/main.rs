use actix_web::{
    App, Either, HttpResponse, HttpServer, Responder, get,
    http::StatusCode,
    rt::task::spawn_blocking,
    web::{self, Data, Json},
};
use clap::Parser;
use depiction_map::{
    DepictAppData, DepictionCategory, FetchDataOpenStreetMap, FetchedDataSet, MapEntry,
};
use log::info;
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

        let mut app_data = DepictAppData::new(&fetched_data_set);
        app_data.start_update_thread(fetched_data_set);
        Data::new(app_data)
    }).await.unwrap();

    info!("Starting server on {}:{}", opts.host, opts.port);

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .service(index)
            .service(static_ressources)
            .service(get_depiction)
    })
    .bind((opts.host, opts.port))
    .unwrap()
    .run()
    .await
    .unwrap();
}
