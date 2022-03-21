use actix_web::{get, middleware::Logger, App, HttpResponse, HttpServer, Responder};
use log::info;

#[macro_use]
extern crate serde_json;
extern crate log;

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
    let address = "0.0.0.0";
    let port = 8080;

    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    info!("Starting Server at {}:{}", address, port);
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::new(r#""%r" %s %Tms"#))
            .service(hello)
            .service(health)
    })
    .bind((address, port))?
    .run()
    .await
}
