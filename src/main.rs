use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

#[macro_use]
extern crate serde_json;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(json!({"healthy": true}).to_string())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(hello).service(health))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
