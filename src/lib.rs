
mod routes;
mod database;
mod core;

use redis::aio::Connection;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use routes::create_routes;


pub async fn init_db(database_url: &str) -> DatabaseConnection{
    let mut opt = ConnectOptions::new(database_url);
    opt.max_connections(100)
        .min_connections(5);

    Database::connect(opt).await.unwrap()
}

pub async fn init_redis(redis_url: &str) -> Connection{
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let con = client.get_async_connection().await.unwrap();
    con
}

pub async fn run(database_url: &str, redis_url: &str, running_port: &str){
    let database = init_db(database_url).await;
    let redis = init_redis(redis_url).await;
    let app = create_routes(database, redis);
    let url = format!("0.0.0.0:{}", running_port);
    axum::Server::bind(&url.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}