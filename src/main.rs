use std::env;
extern crate log;
extern crate tokio;
use aws_sdk_dynamodb::{config::Region, Client};
use ddb::initialize;
use warp::Filter;

mod ddb;
mod filters;
mod handlers;
mod models;

#[tokio::main]
async fn main() {
    if env::var_os("RUST_LOG").is_none() {
        // Set `RUST_LOG=films=debug` to see debug logs,
        // this only shows access logs.
        env::set_var("RUST_LOG", "films=debug");
    }
    pretty_env_logger::init();
    let config = aws_config::from_env()
        .region(Region::new("us-east-1"))
        .load()
        .await;
    let dynamodb_local_config = aws_sdk_dynamodb::config::Builder::from(&config)
        .endpoint_url(
            // 8000 is the default dynamodb port
            "http://localhost:8000",
        )
        .build();

    let db_client = Client::from_conf(dynamodb_local_config);
    let _ = db_client.delete_table().table_name("films").send().await;

    let _ = initialize(&db_client, "films").await;
    let api = filters::films(db_client);

    // View access logs by setting `RUST_LOG=films`.
    let routes = api.with(warp::log("films-api"));
    // Start up the server...
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}

//Tests
#[cfg(test)]
mod tests;
