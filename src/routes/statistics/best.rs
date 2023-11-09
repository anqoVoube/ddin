use axum::Extension;
use scylla::Session as ScyllaDBSession;
use crate::core::auth::middleware::Auth;

pub async fn get_best(
    Extension(scylla): Extension<ScyllaDBSession>,
    Extension(Auth {user_id, business_id}): Extension<Auth>
){
    let start_date = "2023-11-01 00:00:00";
    let end_date = "2023-11-10 00:00:00";
    let query = "SELECT parent_id, SUM(quantity) FROM statistics.products WHERE business_id = ? AND date >= ? AND date <= ? GROUP BY parent_id, business_id";

    let results = scylla.query(
        query,
        (business_id, start_date, end_date)
    ).await?;

    let mut max_parent_id = 0;
    let mut max_quantity = 0;
    for row in results.rows.into_typed::<(i32, i32)>() {
        let parent_id: i32 = row.get("parent_id")?;
        let quantity_sum: i32 = row.get("system.sum(quantity)")?;
        if quantity_sum > max_quantity{
            max_parent_id = parent_id;
            max_quantity = quantity_sum;
        }
    }

}


use scylla::{IntoTypedRows, Session, SessionBuilder};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Create a new Session which connects to node at 127.0.0.1:9042
    // (or SCYLLA_URI if specified)
    let uri = std::env::var("SCYLLA_URI")
        .unwrap_or_else(|_| "127.0.0.1:9042".to_string());

    let session: Session = SessionBuilder::new()
        .known_node(uri)
        .build()
        .await?;

    let a = session.clone();

    // Create an example keyspace and table
    a
        .query(
            "CREATE KEYSPACE IF NOT EXISTS statistics WITH REPLICATION = \
            {'class' : 'NetworkTopologyStrategy', 'replication_factor' : 1}",
            &[],
        )
        .await?;

    println!("HELLO");
    a
        .query(
            "CREATE TABLE IF NOT EXISTS statistics.products (
                parent_id int,
                quantity int,
                business_id int,
                date timestamp,
                PRIMARY KEY ((parent_id, business_id), date)
            );",
            &[],
        )
        .await?;

    println!("HELLO 2");
    a
        .query("INSERT INTO statistics.products (parent_id, quantity, business_id, date) VALUES (2, 100, 100, '2023-11-06 00:00:00');

", &[]).await?;

// INSERT INTO statistics.products (parent_id, quantity, business_id, date) VALUES (2, 7, 100, '2023-11-13 00:00:00');
// INSERT INTO statistics.products (parent_id, quantity, business_id, date) VALUES (2, 7, 100, '2023-11-14 00:00:00');
// INSERT INTO statistics.products (parent_id, quantity, business_id, date) VALUES (2, 8, 100, '2023-11-12 00:00:00');
// INSERT INTO statistics.products (parent_id, quantity, business_id, date) VALUES (2, 20, 100, '2023-11-18 00:00:00');

    println!("HELLO 3");
    // Query rows from the table and print them


    Ok(())
}