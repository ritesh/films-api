use aws_sdk_dynamodb::{config::Region, Client};
use warp::http::StatusCode;
use warp::test::request;

use super::{filters, models::Film};

#[tokio::test]
async fn test_welcome() {
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

    let dbclient = Client::from_conf(dynamodb_local_config);
    let api = filters::films(dbclient);
    let resp = request().method("GET").path("/").reply(&api).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_post() {
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

    let dbclient = Client::from_conf(dynamodb_local_config);
    let api = filters::films(dbclient);

    let resp = request()
        .method("POST")
        .path("/films")
        .json(&Film {
            year: 2000,
            title: "Coool film".into(),
            genres: vec!["foo".into()],
            cast: vec!["foo".into()],
            href: Some("".into()),
            thumbnail: Some("".into()),
            thumbnail_width: Some(2),
            thumbnail_height: Some(4),
            extract: Some("blah blah".into()),
        })
        .reply(&api)
        .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
}

// #[tokio::test]
// async fn test_post_conflict() {
//     let db = models::blank_db();
//     db.lock().await.push(film1());
//     let api = filters::films(db);

//     let resp = request()
//         .method("POST")
//         .path("/films")
//         .json(&film1())
//         .reply(&api)
//         .await;

//     assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
// }

// fn film1() -> Film {
//     let genres = vec!["Comedy".into(), "Horror".into()];

//     Film {
//         title: "Some film".into(),
//         year: 2020,
//         cast: vec!["Person One".into(), "Person Two".into()],
//         genres: genres,
//         href: Some("Some_film".into()),
//         extract: Some("This is a dummy film".into()),
//         thumbnail: Some("http://example.com/1.jpg".into()),
//         thumbnail_width: Some(200),
//         thumbnail_height: Some(327),
//     }
// }
