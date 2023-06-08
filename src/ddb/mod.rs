use crate::models::Film;
use aws_sdk_dynamodb::{
    operation::create_table::builders::CreateTableFluentBuilder,
    types::{
        AttributeDefinition, KeySchemaElement, KeyType, ProvisionedThroughput, ScalarAttributeType,
        TableStatus, WriteRequest,
    },
    Client,
};
use futures::future::join_all;
use std::{collections::HashMap, time::Duration};
use tracing::{debug, info, trace};
mod error;
const CAPACITY: i64 = 10;

#[tracing::instrument(level = "trace")]
pub async fn initialize(client: &Client, table_name: &str) -> Result<(), error::Error> {
    info!("Initializing Films DynamoDB in {table_name}");

    if table_exists(client, table_name).await? {
        info!("Found existing table {table_name}. Not attempting to bulk load data");
    } else {
        info!("Table does not exist, creating {table_name}");
        create_table(client, table_name, "year", "title", CAPACITY)
            .send()
            .await?;
        await_table(client, table_name).await?;
        bulk_load_data(client, table_name).await?
    }

    Ok(())
}

#[tracing::instrument(level = "trace")]
// Does table exist?
pub async fn table_exists(client: &Client, table: &str) -> Result<bool, error::Error> {
    debug!("Checking for table: {table}");
    let table_list = client.list_tables().send().await;

    match table_list {
        Ok(list) => Ok(list.table_names().as_ref().unwrap().contains(&table.into())),
        Err(e) => Err(e.into()),
    }
}

#[tracing::instrument(level = "trace")]
pub fn create_table(
    client: &Client,
    table_name: &str,
    primary_key: &str,
    sort_key: &str,
    capacity: i64,
) -> CreateTableFluentBuilder {
    info!("Creating table: {table_name} with capacity {capacity} and key structure {primary_key}:{sort_key}");
    client
        .create_table()
        .table_name(table_name)
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name(primary_key)
                .key_type(KeyType::Hash)
                .build(),
        )
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name(primary_key)
                .attribute_type(ScalarAttributeType::N)
                .build(),
        )
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name(sort_key)
                .key_type(KeyType::Range)
                .build(),
        )
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name(sort_key)
                .attribute_type(ScalarAttributeType::S)
                .build(),
        )
        .provisioned_throughput(
            ProvisionedThroughput::builder()
                .read_capacity_units(capacity)
                .write_capacity_units(capacity)
                .build(),
        )
}

const TABLE_WAIT_POLLS: u64 = 6;
const TABLE_WAIT_TIMEOUT: u64 = 10;
pub async fn await_table(client: &Client, table_name: &str) -> Result<(), error::Error> {
    // TODO: Use an adaptive backoff retry, rather than a sleeping loop.
    for _ in 0..TABLE_WAIT_POLLS {
        debug!("Checking if table is ready: {table_name}");
        if let Some(table) = client
            .describe_table()
            .table_name(table_name)
            .send()
            .await?
            .table()
        {
            if matches!(table.table_status, Some(TableStatus::Active)) {
                debug!("Table is ready");
                return Ok(());
            } else {
                debug!("Table is NOT ready")
            }
        }
        tokio::time::sleep(Duration::from_secs(TABLE_WAIT_TIMEOUT)).await;
    }

    Err(error::Error::table_not_ready(table_name))
}

// Must be less than 26.
const CHUNK_SIZE: usize = 25;

pub async fn bulk_load_data(client: &Client, table_name: &str) -> Result<(), error::Error> {
    debug!("Loading data into table {table_name}");
    let data: Vec<Film> =
        serde_json::from_str(include_str!("./t.json")).expect("loading large Films dataset");

    let data_size = data.len();
    trace!("Loading {data_size} items in batches of {CHUNK_SIZE}");

    let ops = data
        .iter()
        .map(|v| {
            WriteRequest::builder()
                .set_put_request(Some(v.into()))
                .build()
        })
        .collect::<Vec<WriteRequest>>();

    let batches = ops
        .chunks(CHUNK_SIZE)
        .map(|chunk| write_batch(client, table_name, chunk));
    let batches_count = batches.len();

    trace!("Awaiting batches, count: {batches_count}");
    join_all(batches).await;

    Ok(())
}

pub async fn write_batch(
    client: &Client,
    table_name: &str,
    ops: &[WriteRequest],
) -> Result<(), error::Error> {
    assert!(
        ops.len() <= 25,
        "Cannot write more than 25 items in a batch"
    );
    let mut unprocessed = Some(HashMap::from([(table_name.to_string(), ops.to_vec())]));
    while unprocessed_count(unprocessed.as_ref(), table_name) > 0 {
        let count = unprocessed_count(unprocessed.as_ref(), table_name);
        trace!("Adding {count} unprocessed items");
        unprocessed = client
            .batch_write_item()
            .set_request_items(unprocessed)
            .send()
            .await?
            .unprocessed_items;
    }

    Ok(())
}

fn unprocessed_count(
    unprocessed: Option<&HashMap<String, Vec<WriteRequest>>>,
    table_name: &str,
) -> usize {
    unprocessed
        .map(|m| m.get(table_name).map(|v| v.len()).unwrap_or_default())
        .unwrap_or_default()
}
