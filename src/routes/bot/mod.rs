// use std::convert::Infallible;
// use std::future::Future;
// use axum::response::{IntoResponse, Response};
// use axum::Router;
// use teloxide::update_listeners::UpdateListener;
// use teloxide::update_listeners::webhooks::Options;
//
// pub fn bot_webhook(
//     options: Options
// ) -> Result<(impl UpdateListener<Err = Infallible>, impl Future<Output = ()>, Router), Response>{
//     println!("HELLO!");
//     Err(().into_response())
// }