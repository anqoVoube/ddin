use dotenvy::dotenv;
use updddin::run;


use teloxide::prelude::*;


#[tokio::main]
async fn main() {
    dotenv().ok();
    println!("RecompileS!!!");
    run().await;
}


// This example demonstrates how to deal with messages and callback queries
// within a single dialogue.
//
// # Example
// ```
// - /start
// - Let's start! What's your full name?
// - John Doe
// - Select a product:
//   [Apple, Banana, Orange, Potato]
// - <A user selects "Banana">
// - John Doe, product 'Banana' has been purchased successfully!
// ```}