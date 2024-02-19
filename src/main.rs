use dotenvy::dotenv;
use updddin::run;


use teloxide::prelude::*;


#[tokio::main]
async fn main() {
    dotenv().ok();
    run().await;
}
