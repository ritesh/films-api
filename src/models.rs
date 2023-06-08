use std::collections::HashMap;

use aws_sdk_dynamodb::types::{AttributeValue, PutRequest};
use aws_smithy_client::SdkError;
use serde::{Deserialize, Serialize};
//use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FilmError {
    // #[error("failed to parse serde_json::Value into Film {0}")]
    // FromValue(&'static Value),
    #[error("failed to parse response into films: {0}")]
    FromSerde(serde_dynamo::Error),

    #[error("aws_sdk_dynamodb error: {0}")]
    Dynamo(aws_sdk_dynamodb::Error),

    #[error("unknown DynamoDB films error: {0}")]
    Unknown(String),
}

impl From<aws_sdk_dynamodb::Error> for FilmError {
    fn from(err: aws_sdk_dynamodb::Error) -> Self {
        FilmError::Dynamo(err)
    }
}

impl From<serde_dynamo::Error> for FilmError {
    fn from(err: serde_dynamo::Error) -> Self {
        FilmError::FromSerde(err)
    }
}

impl<E> From<SdkError<E>> for FilmError
where
    E: std::fmt::Debug,
{
    fn from(err: SdkError<E>) -> Self {
        FilmError::Unknown(format!("{err:?}"))
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Film {
    pub year: i32,
    pub title: String,
    #[serde(default = "Vec::new")]
    pub genres: Vec<String>,
    #[serde(alias = "actors", default = "Vec::new")]
    pub cast: Vec<String>,
    pub href: Option<String>,
    pub thumbnail: Option<String>,
    pub thumbnail_width: Option<i32>,
    pub thumbnail_height: Option<i32>,
    pub extract: Option<String>,
}

impl Film {
    pub fn new(year: i32, title: String) -> Self {
        Film {
            year,
            title,
            genres: Vec::new(),
            cast: Vec::new(),
            href: None,
            thumbnail: None,
            thumbnail_width: None,
            thumbnail_height: None,
            extract: None,
        }
    }

    pub fn cast_mut(&mut self) -> &mut Vec<String> {
        &mut self.cast
    }
    pub fn genres_mut(&mut self) -> &mut Vec<String> {
        &mut self.genres
    }
    pub fn thumbnail_mut(&mut self) -> &mut Option<String> {
        &mut self.thumbnail
    }
    pub fn thumbnail_height(&mut self) -> &mut Option<i32> {
        &mut self.thumbnail_height
    }
    pub fn thumbnail_width(&mut self) -> &mut Option<i32> {
        &mut self.thumbnail_width
    }
}

fn as_string(val: Option<&AttributeValue>, default: &String) -> String {
    if let Some(v) = val {
        if let Ok(s) = v.as_s() {
            return s.to_owned();
        }
    }
    default.to_owned()
}

fn as_i32(val: Option<&AttributeValue>, default: i32) -> i32 {
    if let Some(v) = val {
        if let Ok(n) = v.as_n() {
            if let Ok(n) = n.parse::<i32>() {
                return n;
            }
        }
    }
    default
}

fn as_string_vec(val: Option<&AttributeValue>) -> Vec<String> {
    if let Some(val) = val {
        if let Ok(val) = val.as_l() {
            return val
                .iter()
                .map(|v| as_string(Some(v), &"".to_string()))
                .collect();
        }
    }
    vec![]
}

impl From<&HashMap<String, AttributeValue>> for Film {
    fn from(value: &HashMap<String, AttributeValue>) -> Self {
        let mut film = Film::new(
            as_i32(value.get("year"), 0),
            as_string(value.get("title"), &"".to_string()),
        );

        let mut genres: Vec<String> = as_string_vec(value.get("genres"));
        let mut cast: Vec<String> = as_string_vec(value.get("cast"));

        film.genres_mut().append(&mut genres);
        film.cast_mut().append(&mut cast);

        film
    }
}

impl From<&Film> for PutRequest {
    fn from(film: &Film) -> Self {
        PutRequest::builder()
            .item("year", AttributeValue::N(film.year.to_string()))
            .item("title", AttributeValue::S(film.title.clone()))
            .item(
                "cast",
                AttributeValue::L(
                    film.cast
                        .iter()
                        .map(|v| AttributeValue::S(v.clone()))
                        .collect(),
                ),
            )
            .item(
                "genres",
                AttributeValue::L(
                    film.genres
                        .iter()
                        .map(|v| AttributeValue::S(v.clone()))
                        .collect(),
                ),
            )
            .item(
                "href",
                AttributeValue::S(film.href.clone().unwrap_or("default".into())),
            )
            .item(
                "thumbnail",
                AttributeValue::S(film.thumbnail.clone().unwrap_or("default".into())),
            )
            .item(
                "thumbnail_height",
                AttributeValue::N(film.thumbnail_height.unwrap_or(0).to_string()),
            )
            .item(
                "thumbnail_width",
                AttributeValue::N(film.thumbnail_width.unwrap_or(0).to_string()),
            )
            .item(
                "extract",
                AttributeValue::S(film.extract.clone().unwrap_or("".into())),
            )
            .build()
    }
}

#[cfg(test)]
mod test {
    use aws_sdk_dynamodb::types::PutRequest;

    use super::Film;

    #[test]
    fn test_put_request_from_film_and_back() {
        let mut film = Film::new(2022, "Knives Out".into());
        film.cast_mut().append(&mut vec![
            "Daniel Craig".into(),
            "Edward Norton".into(),
            "Janelle Monae".into(),
        ]);
        film.genres_mut()
            .append(&mut vec!["Mystery".into(), "Comedy".into()]);

        let request: PutRequest = (&film).into();

        let item = request.item().unwrap();
        assert_eq!(item.len(), 9);

        let film_back: Film = item.into();

        assert_eq!(film_back, film);
    }
}

//FixedResponse returns a welcome message
#[derive(Serialize, Deserialize, Debug)]
pub struct FixedResponse {
    pub status: String,
    pub remote_address: String,
}

// The query parameters for list films.
#[derive(Debug, Deserialize)]
pub struct ListOptions {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
    pub title: Option<String>,
    pub genre: Option<String>,
    pub year: Option<u16>,
}
