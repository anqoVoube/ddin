
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

    Database::connect(opt).await.expect("Failed to create Postgres connection")
}

pub async fn init_redis(redis_uri: &str) -> RedisPool {
    let manager = match bb8_redis::RedisConnectionManager::new(redis_uri) {
        Ok(manager) => manager,
        Err(e) => {
            error!("Failed to create Redis connection manager: {}", e);
            panic!("Redis Error: {}", e);
        }
    };
    bb8::Pool::builder()
        .build(manager)
        .await
        .expect("Failed to create Redis pool.")
}


pub async fn init_scylla(scylla_url: &str) -> Session{
   let session = SessionBuilder::new()
       .known_node(scylla_url)
       .build()
       .await
       .expect("Failed to create ScyllaDB session");

    // Keyspace and Table creation
    session
        .query(
            "CREATE KEYSPACE IF NOT EXISTS statistics WITH REPLICATION = \
            {'class' : 'NetworkTopologyStrategy', 'replication_factor' : 1}",
            &[],
        )
        .await
        .expect("Failed to create keyspace");

    session
        .query(
            "CREATE TABLE IF NOT EXISTS statistics.products (
                parent_id int,
                item_type tinyint,
                quantity int,
                profit int,
                business_id int,
                date date,
                PRIMARY KEY ((parent_id, business_id, item_type), date)
            );",
            &[],
        )
        .await
        .expect("Failed to create table products");

    session
        .query(
            "CREATE TABLE IF NOT EXISTS statistics.profits (
                business_id int,
                profit int,
                date date,
                PRIMARY KEY ((date, business_id))
            );",
            &[],
        )
        .await
        .expect("Failed to create table products");

    session
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
        .expect("Failed to run axum server");
}