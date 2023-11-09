
mod routes;
mod database;
mod core;

use log::error;
use scylla::{IntoTypedRows, Session, SessionBuilder, SessionConfig};
use redis::aio::Connection;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use routes::create_routes;

type RedisPool = bb8::Pool<bb8_redis::RedisConnectionManager>;

pub async fn init_db(database_uri: &str) -> DatabaseConnection{
    let mut opt = ConnectOptions::new(database_uri);
    opt.max_connections(100)
        .min_connections(5);

    Database::connect(opt).await.unwrap()
}

pub async fn init_redis(redis_uri: &str) -> RedisPool {
    let manager = bb8_redis::RedisConnectionManager::new(redis_uri).unwrap();
    bb8::Pool::builder()
        .build(manager)
        .await
        .expect("Failed to create Redis pool.")
}


pub async fn init_scylla(scylla_url: &str) -> Session{
    match SessionBuilder::new()
        .known_node(scylla_url)
        .build()
        .await{
            Ok(session) => session,
            Err(err) => {
                error!("Failed to initialize scylaldb");
                panic!("Failed to initialize");
            }
        }
}

pub async fn run(database_uri: &str, redis_uri: &str, scylla_uri: &str, running_port: &str){
    let database = init_db(database_uri).await;
    let redis = init_redis(redis_uri).await;
    let scylla = init_scylla(scylla_uri).await;
    let app = create_routes(database, redis, scylla);
    let url = format!("0.0.0.0:{}", running_port);
    axum::Server::bind(&url.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}