
use actix_cors::Cors;
use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::collections::HashMap;
use reqwest::Client;
use async_trait::async_trait;

#[derive(Debug, Deserialize, Serialize)]
struct FitnessProgress {
    user_id: String,
    progress: f32,
    timestamp: i64,
    timezone: String,
}

struct AppState {
    fitness_progress: Mutex<HashMap<String, FitnessProgress>>,
    http_client: Client,
}

async fn track_progress(data: web::Data<AppState>, progress: web::Json<FitnessProgress>) -> impl Responder {
    let mut fitness_progress = data.fitness_progress.lock().unwrap();

    fitness_progress.insert(progress.user_id.clone(), progress.into_inner());

    HttpResponse::Ok().body("Fitness progress tracked")
}

async fn fetch_progress(data: web::Data<AppState>, user_id: web::Path<String>) -> impl Responder {
    let fitness_progress = data.fitness_progress.lock().unwrap();

    if let Some(progress) = fitness_progress.get(&user_id.into_inner()) {
        HttpResponse::Ok().json(progress)
    } else {
        HttpResponse::NotFound().body("No progress found for this user")
    }
}

#[async_trait]
trait TimeZone {
    async fn get_time_zone(&self, timezone: &str) -> Result<String, reqwest::Error>;
}

#[derive(Clone)]
struct TimeZoneClient {
    client: Client,
}

#[async_trait]
impl TimeZone for TimeZoneClient {
    async fn get_time_zone(&self, timezone: &str) -> Result<String, reqwest::Error> {
        let url = format!("http://worldtimeapi.org/api/timezone/{}", timezone);
        let resp = self.client.get(&url).send().await?;
        let body = resp.text().await?;
        Ok(body)
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let fitness_progress = Mutex::new(HashMap::new());
    let http_client = Client::new();
    let time_zone_client = TimeZoneClient { client: http_client.clone() };

    HttpServer::new(move || {
        App::new()
            .data(AppState { fitness_progress: fitness_progress.clone(), http_client: http_client.clone() })
            .wrap(Cors::permissive())
            .route("/track_progress", web::post().to(track_progress))
            .route("/fetch_progress/{user_id}", web::get().to(fetch_progress))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
