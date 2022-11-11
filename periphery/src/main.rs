#![allow(unused)]

use axum::{extract::Path, http::StatusCode, routing::get, Extension, Json, Router};

mod api;
mod config;
mod helpers;

use crate::helpers::get_socket_addr;

use api::*;

#[tokio::main]
async fn main() {
    let (port, secrets) = config::load();

    let app = api::router();

    println!("starting montior periphery on port {port}");

    axum::Server::bind(&get_socket_addr(port))
        .serve(app.into_make_service())
        .await
        .expect("monitor periphery axum server crashed");
}
