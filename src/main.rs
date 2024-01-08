use dotenvy_macro::dotenv;
use dotenvy::dotenv;
use updddin::run;


#[tokio::main]
async fn main() {
    dotenv().ok();
    let redis_url = dotenv!("REDIS_URI");
    let scylla_uri = dotenv!("SCYLLA_URI");
    let mongo_uri = dotenv!("MONGO_URI");
    let running_port = dotenv!("API_PORT");
    run(redis_url, scylla_uri, mongo_uri, running_port).await;
}
