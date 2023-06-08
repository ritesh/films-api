use crate::models::{Film, FixedResponse, ListOptions};
use aws_sdk_dynamodb::{
    types::{AttributeValue, PutRequest},
    Client,
};
use std::convert::Infallible;
use tokio_stream::StreamExt;
use warp::http::StatusCode;

pub async fn welcome(addr: Option<String>) -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::json(&FixedResponse {
        status: StatusCode::OK.to_string(),
        remote_address: String::from(addr.unwrap_or("unknown".into())),
    }))
}

pub async fn list_films(
    opts: ListOptions,
    dbclient: Client,
) -> Result<impl warp::Reply, Infallible> {
    log::debug!("List films {:?}", opts);
    match opts.year {
        Some(y) => {
            log::debug!("Year is {}", y);
            let results = dbclient
                .query()
                .table_name("films")
                .key_condition_expression("#yr = :yyyy")
                .expression_attribute_names("#yr", "year")
                .expression_attribute_values(":yyyy", AttributeValue::N(y.to_string()))
                .send()
                .await;
            if let Some(items) = results.unwrap().items {
                let films: Vec<Film> = items.iter().map(|v| v.into()).collect();
                Ok(warp::reply::json(&films))
            } else {
                Ok(warp::reply::json(&FixedResponse {
                    status: StatusCode::NOT_FOUND.to_string(),
                    remote_address: "unknown".into(),
                }))
            }
        }
        //No year found return everything
        None => {
            let items: Result<Vec<_>, _> = dbclient
                .scan()
                .table_name("films")
                .into_paginator()
                .items()
                .send()
                .collect()
                .await;

            if items.is_ok() {
                let mut v: Vec<Film> = Vec::new();
                for i in items {
                    for j in i {
                        let f: Film = (&j).into();
                        v.push(f);
                    }
                }
                return Ok(warp::reply::json(&v));
            }

            Ok(warp::reply::json(&FixedResponse {
                status: StatusCode::NOT_FOUND.to_string(),
                remote_address: "unknown".into(),
            }))
        }
    }
}

pub async fn create_film(create: Film, dbclient: Client) -> Result<impl warp::Reply, Infallible> {
    log::debug!("create_film: {:?}", create);
    let putreq: PutRequest = (&create).into();
    match dbclient
        .put_item()
        .table_name("films")
        .set_item(putreq.item().cloned())
        .send()
        .await
    {
        Ok(_) => return Ok(StatusCode::CREATED),
        Err(e) => {
            log::warn!("Error! {}", e);
            return Ok(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}

// pub async fn query_item(client: &Client, item: Film) -> bool {
//     let value = &item.value;
//     let key = &item.key;
//     let user_av = AttributeValue::S(value.to_string());

//     match client
//         .query()
//         .table_name(item.table)
//         .key_condition_expression("#key = :value".to_string())
//         .expression_attribute_names("#key".to_string(), key.to_string())
//         .expression_attribute_values(":value".to_string(), user_av)
//         .select(Select::AllAttributes)
//         .send()
//         .await
//     {
//         Ok(resp) => {
//             if resp.count > 0 {
//                 println!("Found a matching entry in the table:");
//                 println!("{:?}", resp.items.unwrap_or_default().pop());
//                 true
//             } else {
//                 println!("Did not find a match.");
//                 false
//             }
//         }
//         Err(e) => {
//             println!("Got an error querying table:");
//             println!("{}", e);
//             process::exit(1);
//         }
//     }
// }
// pub async fn update_film(
//     title: String,
//     update: Film,
//     db: Db,
// ) -> Result<impl warp::Reply, Infallible> {
//     // If the for loop didn't return OK, then the ID doesn't exist...
//     Ok(StatusCode::NOT_FOUND)
// }

// pub async fn delete_film(title: String, db: Db) -> Result<impl warp::Reply, Infallible> {
//     let title = decode(title);
//     log::info!("delete_film: id={}", title);

//     let mut vec = db.lock().await;

//     let len = vec.len();
//     vec.retain(|film| {
//         // Retain all films that aren't this id...
//         // In other words, remove all that *are* this id...
//         film.title != title
//     });

//     // If the vec is smaller, we found and deleted a film!
//     let deleted = vec.len() != len;

//     if deleted {
//         // respond with a `204 No Content`, which means successful,
//         // yet no body expected...
//         Ok(StatusCode::NO_CONTENT)
//     } else {
//         log::debug!("    -> film id not found!");
//         Ok(StatusCode::NOT_FOUND)
//     }
// }
