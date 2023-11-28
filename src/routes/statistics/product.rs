use scylla::IntoTypedRows;
use crate::routes::ScyllaDBConnection;
use crate::routes::statistics::{get_date_range, Types};

pub async fn get_product_stats(
    ScyllaDBConnection {scylla}: ScyllaDBConnection,
    parent_id: i32,
    business_id: i32,
    item_type: i8,
    r#type: Types,
    prev: u8
){
    let (start_date, end_date, namings) = get_date_range(r#type, prev);
    let query = "SELECT quantity, profit FROM statistics.products WHERE parent_id = ? AND business_id = ? AND date >= ? AND date <= ? AND item_type = ? ALLOW FILTERING";

    let results = scylla.query(
        query,
        (parent_id, business_id, start_date, end_date, item_type)
    ).await.expect("Failed to query");

    let brah = results.rows.expect("");
    println!("{:?}", brah);
    println!("{}", brah.len());
    for row in brah.into_typed::<(i32, i32)>() {
        if let Ok(result) = row{
            let (quantity, profit) = result;
            println!("quantity: {}, profit: {}", quantity, profit);
        }
    }
}