
use actix_cors::Cors;
use actix_web::{http::header, web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};
use reqwest::Client as HttpClient;
use async_trait::async_trait;
use std::sync::Mutex;
use std::collections::HashMap;
use std::fs;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Event {
    id: u64,
    name: String,
    description: String,
    date: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Database {
    events: HashMap<u64, Event>,
}

impl Database {
    fn new() -> Self {
        Self {
            events: HashMap::new(),
        }
    }

    fn insert(&mut self, event: Event) {
        self.events.insert(event.id, event);
    }

    fn get(&self, id: &u64) -> Option<&Event> {
        self.events.get(id)
    }

    fn get_all(&self) -> Vec<&Event> {
        self.events.values().collect()
    }

    fn delete(&mut self, id: &u64) {
        self.events.remove(id);
    }

    fn update(&mut self, event: Event) {
        self.events.insert(event.id, event);
    }

    fn save_to_file(&self) -> std::io::Result<()> {
        let data: String = serde_json::to_string(&self)?;
        let mut file: fs::File = fs::File::create("database.json")?;
        file.write_all(data.as_bytes())?;
        Ok(())
    }

    fn load_from_file() -> std::io::Result<Self> {
        let file_content: String = fs::read_to_string("database.json")?;
        let db: Database = serde_json::from_str(&file_content)?;
        Ok(db)
    }
}

struct AppState {
    db: Mutex<Database>
}

async fn create_event(app_state: web::Data<AppState>, event: web::Json<Event>) -> impl Responder {
    let mut db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    db.insert(event.into_inner());
    let _ = db.save_to_file();
    HttpResponse::Ok().finish()
}

async fn read_event(app_state: web::Data<AppState>, id: web::Path<u64>) -> impl Responder {
    let db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    match db.get(&id.into_inner()) {
        Some(event) => HttpResponse::Ok().json(event),
        None => HttpResponse::NotFound().finish()
    }
}

async fn read_all_events(app_state: web::Data<AppState>) -> impl Responder {
    let db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    let events = db.get_all();
    HttpResponse::Ok().json(events)
}

async fn update_event(app_state: web::Data<AppState>, event: web::Json<Event>) -> impl Responder {
    let mut db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    db.update(event.into_inner());
    let _ = db.save_to_file();
    HttpResponse::Ok().finish()
}

async fn delete_event(app_state: web::Data<AppState>, id: web::Path<u64>) -> impl Responder {
    let mut db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    db.delete(&id.into_inner());
    let _ = db.save_to_file();
    HttpResponse::Ok().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let db: Database = match Database::load_from_file() {
        Ok(db) => db,
        Err(_) => Database::new()
    };

    let data: web::Data<AppState> = web::Data::new(AppState {
        db: Mutex::new(db)
    });

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::permissive()
                    .allowed_origin_fn(|origin, _req_head| {
                        origin.as_bytes().starts_with(b"http://localhost") || origin == "null"
                    })
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .supports_credentials()
                    .max_age(3600)
            )
            .app_data(data.clone())
            .route("/event", web::post().to(create_event))
            .route("/event", web::get().to(read_all_events))
            .route("/event", web::put().to(update_event))
            .route("/event/{id}", web::get().to(read_event))
            .route("/event/{id}", web::delete().to(delete_event))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
