
pub mod routes;
mod database;
mod core;

use dotenvy_macro::dotenv;

use log::error;
use scylla::{IntoTypedRows, Session, SessionBuilder};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use routes::create_routes;
use mongodb::{
    Client,
};

type RedisPool = bb8::Pool<bb8_redis::RedisConnectionManager>;

pub async fn init_db() -> DatabaseConnection{
    let database_uri = dotenv!("DATABASE_URI");
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
        .expect("Failed to create table products.");

    session
}


pub async fn init_mongo(mongo_uri: &str) -> mongodb::Database{
    let client = Client::with_uri_str(mongo_uri).await.expect("Failed to create MongoDB client");
    client.database("history")
}

pub async fn init_barcode_sqlite() -> rusqlite::Connection{
    rusqlite::Connection::open("barcodes.db").expect("Failed to open database")
}

pub async fn run(redis_uri: &str, scylla_uri: &str, mongo_uri: &str, running_port: &str){
    let database = init_db().await;
    let redis = init_redis(redis_uri).await;
    let scylla = init_scylla(scylla_uri).await;
    let mongo = init_mongo(mongo_uri).await;
    let sqlite = init_barcode_sqlite().await;
    let app = create_routes(database, redis, scylla, mongo, sqlite);
    let url = format!("0.0.0.0:{}", running_port);
    axum::Server::bind(&url.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .expect("Failed to run axum server");
}