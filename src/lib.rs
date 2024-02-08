
pub mod routes;
mod database;
mod core;
pub mod bot;

use axum::Router;
use dotenvy_macro::dotenv;

use log::error;
use scylla::{IntoTypedRows, Session, SessionBuilder};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use routes::create_routes;
use mongodb::{
    Client,
};
use teloxide::{Bot, dptree};
use teloxide::prelude::{Message, Requester, Update};
use teloxide::update_listeners::webhooks::{axum_no_setup, Options};
use once_cell::sync::OnceCell;
use teloxide::payloads::SendMessageSetters;

use teloxide::{
    dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler},
    prelude::*,
    utils::command::BotCommands,
};

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
type RedisPool = bb8::Pool<bb8_redis::RedisConnectionManager>;

static POSTGRES_CONNECTION: OnceCell<DatabaseConnection> = OnceCell::new();


#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    // ReceiveAge {
    //     full_name: String,
    // },
    // ReceiveLocation {
    //     full_name: String,
    //     age: u8,
    // },
    ReceiveContact,
    ChooseBusiness,
}


pub async fn init_db() -> DatabaseConnection{
    let frick = dotenv!("DATABASE_URI");
    println!("{}", frick);
    let mut opt = ConnectOptions::new(
        frick
    );
    opt.max_connections(100)
        .min_connections(5);

    Database::connect(opt).await.expect("Failed to create Postgres connection")
}

pub async fn init_redis() -> RedisPool {
    let manager = match bb8_redis::RedisConnectionManager::new(
        dotenv!("REDIS_URI")
    ) {
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


pub async fn init_scylla() -> Session{
   let session = SessionBuilder::new()
       .known_node(
           dotenv!("SCYLLA_URI")
       )
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


pub async fn init_mongo() -> mongodb::Database{
    let client = Client::with_uri_str(
        dotenv!("MONGO_URI")
    ).await.expect("Failed to create MongoDB client");
    client.database("history")
}

pub async fn init_barcode_sqlite() -> rusqlite::Connection{
    rusqlite::Connection::open(
        dotenv!("SQLITE_DB_NAME")
    ).expect("Failed to open database")
}


pub async fn init_bot() -> Router{
    let bot = Bot::new(dotenv!("BOT_TOKEN"));
    let options = Options {
        address: ([0, 0, 0, 0], 3000).into(),
        url: "https://ddin.uz/webhook".parse().unwrap(),
        certificate: None,
        max_connections: None,
        drop_pending_updates: false,
        secret_token: None,
    };

    let (
        listener,
        stop_flag,
        router
    ) = axum_no_setup(
        options
    );

    fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
        use dptree::case;


        let message_handler = Update::filter_message()
            .enter_dialogue::<Message, InMemStorage<State>, State>()
            .branch(dptree::case![State::Start].endpoint(bot::start))
            .branch(dptree::case![State::ReceiveContact].endpoint(bot::receive_full_name));

        let callback_query_handler = Update::filter_callback_query()
            .branch(case![State::ChooseBusiness])
            .endpoint(bot::handle_callback_query);

        dialogue::enter::<Update, InMemStorage<State>, State, _>()
            .branch(message_handler)
            .branch(callback_query_handler)
    }

    tokio::spawn(async move {
        Dispatcher::builder(
            bot,
            schema()
        )
            .dependencies(dptree::deps![InMemStorage::<State>::new()])
            .build()
            // .dispatch()
            .dispatch_with_listener(
                listener,
                LoggingErrorHandler::with_custom_text("An error from the update listener"),
            )
            .await;
        }
    );
    router
}

pub async fn run(){
    let database = init_db().await;
    // DB Connection For Telegram Bot
    POSTGRES_CONNECTION.set(database.clone()).unwrap();
    let redis = init_redis().await;
    let scylla = init_scylla().await;
    let mongo = init_mongo().await;
    let sqlite = init_barcode_sqlite().await;
    let bot_axum_router = init_bot().await;
    log::info!("Starting dialogue bot..");

    let app = create_routes(database, redis, scylla, mongo, sqlite, bot_axum_router);
    let url = format!("0.0.0.0:{}", dotenv!("API_PORT"));

    axum::Server::bind(&url.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .expect("Failed to run axum server");
}