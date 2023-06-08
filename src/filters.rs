use super::models::{Film, ListOptions};
use crate::handlers;

use aws_sdk_dynamodb::Client;
use warp::Filter;

pub fn welcome() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let host = warp::header::optional::<String>("host");
    warp::path::end()
        .and(warp::get())
        .and(host)
        .and_then(handlers::welcome)
}

/// GET /films
pub fn films(
    dbclient: Client,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    welcome()
        .or(films_list(dbclient.clone()))
        .or(films_create(dbclient.clone()))
    //     .or(films_update(dbclient.clone()))
    //    .or(films_delete(dbclient.clone()))
}

/// GET /films?offset=3&limit=5
pub fn films_list(
    dbclient: Client,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("films")
        .and(warp::get())
        .and(warp::query::<ListOptions>())
        .and(with_db(dbclient))
        .and_then(handlers::list_films)
}

/// POST /films with JSON body
pub fn films_create(
    dbclient: Client,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("films")
        .and(warp::post())
        .and(json_body())
        .and(with_db(dbclient))
        .and_then(handlers::create_film)
}

/// PUT /films/title with JSON body
// pub fn films_update(
//     dbclient: Client,
// ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
//     warp::path!("films" / String)
//         .and(warp::put())
//         .and(json_body())
//         .and(with_db(dbclient))
//         .and_then(handlers::update_film)
// }

/// DELETE /films/title
// pub fn films_delete(
//     dbclient: Client,
// ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
//     warp::path!("films" / String)
//         .and(warp::delete())
//         .and(with_db(dbclient))
//         .and_then(handlers::delete_film)
// }

fn with_db(
    dbclient: Client,
) -> impl Filter<Extract = (Client,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || dbclient.clone())
}

fn json_body() -> impl Filter<Extract = (Film,), Error = warp::Rejection> + Clone {
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}
