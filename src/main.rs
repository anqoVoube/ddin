use dotenvy_macro::dotenv;
use dotenvy::dotenv;
use updddin::run;


#[tokio::main]
async fn main() {
    dotenv().ok();
    let database_uri = dotenv!("DATABASE_URI");
    let redis_url = dotenv!("REDIS_URI");
    let scylla_uri = dotenv!("SCYLLA_URI");
    let running_port = dotenv!("API_PORT");
    run(database_uri, redis_url, scylla_uri, running_port).await;
}
