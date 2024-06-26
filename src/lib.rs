
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
    CallbackHandler,
    ReceiveFirstName {
        lang: String
    },
    ReceiveLastName {
        lang: String,
        first_name: String,
    },
    ReceiveContact {
        lang: String,
        first_name: String,
        last_name: String,
    },
}


pub async fn init_db() -> DatabaseConnection{
    let frick = dotenv!("DATABASE_URI");
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

pub async fn init_mongo() -> mongodb::Database{
    let client = Client::with_uri_str(
        dotenv!("MONGO_URI")
    ).await.expect("Failed to create MongoDB client");
    client.database("history")
}

pub async fn init_barcode_sqlite() -> rusqlite::Connection{
    println!("Connection established");
    rusqlite::Connection::open(
        dotenv!("SQLITE_DB_NAME")
    ).expect("Failed to open database")
}


pub async fn init_bot() -> Bot{
    let bot = Bot::new(dotenv!("BOT_TOKEN"));
    // let options = Options {
    //     address: ([0, 0, 0, 0], 3000).into(),
    //     url: "https://ddin.uz/webhook".parse().unwrap(),
    //     certificate: None,
    //     max_connections: None,
    //     drop_pending_updates: false,
    //     secret_token: None,
    // };
    //
    // let (
    //     listener,
    //     stop_flag,
    //     router
    // ) = axum_no_setup(
    //     options
    // );

    fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
        use dptree::case;

        let message_handler = Update::filter_message()
            .enter_dialogue::<Message, InMemStorage<State>, State>()
            .branch(case![State::Start].endpoint(bot::start))
            .branch(case![State::ReceiveFirstName {lang}].endpoint(bot::receive_first_name))
            .branch(case![State::ReceiveLastName {lang, first_name}].endpoint(bot::receive_last_name))
            .branch(case![State::ReceiveContact {lang, first_name, last_name}].endpoint(bot::receive_contacts));

        let callback_query_handler = Update::filter_callback_query()
            .branch(case![State::CallbackHandler])
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
            .dispatch()
            // .dispatch_with_listener(
            //     listener,
            //     LoggingErrorHandler::with_custom_text("An error from the update listener"),
            // )
            .await;
        }
    );
    Bot::new(dotenv!("BOT_TOKEN"))
}

pub async fn run(){
    let database = init_db().await;
    // DB Connection For Telegram Bot
    POSTGRES_CONNECTION.set(database.clone()).unwrap();
    let redis = init_redis().await;
    let mongo = init_mongo().await;
    let sqlite = init_barcode_sqlite().await;
    let bot_axum_router = init_bot().await;
    log::info!("Starting dialogue bot..");

    let app = create_routes(database, redis, mongo, sqlite, bot_axum_router);
    let url = format!("0.0.0.0:{}", dotenv!("API_PORT"));

    axum::Server::bind(&url.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .expect("Failed to run axum server");
}
