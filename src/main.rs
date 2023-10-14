use dotenvy_macro::dotenv;
use dotenvy::dotenv;
use updddin::run;


#[tokio::main]
async fn main() {
    dotenv().ok();
    let database_uri = dotenv!("DATABASE_URL");
    let running_port = dotenv!("API_PORT");
    run(database_uri, running_port).await;
}
