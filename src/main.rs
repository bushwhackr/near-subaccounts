use actix_web::{get, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use log::info;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::collections::HashMap;

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

#[get("/query/{net}/{account}")]
async fn query(
    path: web::Path<(String, String)>,
    pools: web::Data<HashMap<&str, PgPool>>,
) -> impl Responder {
    let path = path.into_inner();
    let network = path.0.as_str();
    let top_acct = path.1.as_str();

    let pool = pools.get(network);
    if pool.is_none() {
        return HttpResponse::BadRequest().body(format!(
            "invalid network selected: {}. Valid options (testnet, mainnet)",
            network
        ));
    }

    let query_str = format!(
        "
SELECT account_id, created_by_receipt_id, deleted_by_receipt_id, last_update_block_height
FROM accounts
WHERE account_id LIKE '%.{}.{}'",
        top_acct, network
    );

    // Have to create an intermediary object as json! macro cannot deserialize i128/u128
    #[derive(Deserialize, Serialize)]
    struct Account {
        account_id: String,
        created_by_receipt_id: Option<String>,
        deleted_by_receipt_id: Option<String>,
        last_update_block_height: Option<i128>,
    }

    let query_result = sqlx::query(&query_str)
        .map(|row: PgRow| -> Account {
            let account_id: String = row.get("account_id");
            let created_by_receipt_id: Option<String> = row.get("created_by_receipt_id");
            let deleted_by_receipt_id: Option<String> = row.get("deleted_by_receipt_id");
            let last_update_block_height: Option<Decimal> = row.get("last_update_block_height");
            let last_update_block_height = last_update_block_height.map_or(None, |d| d.to_i128());

            Account {
                account_id,
                created_by_receipt_id,
                deleted_by_receipt_id,
                last_update_block_height,
            }
        })
        .fetch_all(pool.unwrap())
        .await;

    return match query_result {
        Ok(rows) => HttpResponse::Ok().json(rows),
        Err(e) => HttpResponse::BadRequest().body(format!("unable to run sql stmt {:?}", e)),
    };
}

async fn init_pools() -> HashMap<&'static str, PgPool> {
    let testnet_pool = PgPoolOptions::new()
        .max_connections(10)
        .connect("postgres://public_readonly:nearprotocol@testnet.db.explorer.indexer.near.dev/testnet_explorer");

    let mainnet_pool = PgPoolOptions::new()
        .max_connections(10)
        .connect("postgres://public_readonly:nearprotocol@mainnet.db.explorer.indexer.near.dev/mainnet_explorer");

    HashMap::from([
        ("testnet", testnet_pool.await.unwrap()),
        ("mainnet", mainnet_pool.await.unwrap()),
    ])
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let address = "0.0.0.0";
    let port = 8080;
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    info!("Initializing Postgres Connection");
    let pools = web::Data::new(init_pools().await);

    info!("Starting Server at {}:{}", address, port);
    HttpServer::new(move || {
        App::new()
            .app_data(pools.clone())
            .wrap(Logger::new(r#""%r" %s %Tms"#))
            .service(hello)
            .service(health)
            .service(query)
    })
    .bind((address, port))?
    .run()
    .await
}
