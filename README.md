# Films API 
This repository contains work-in-progress code for a film query API written in the [Rust Programming Language](https://www.rust-lang.org "Rust programming language"). It uses film data from the [Wikipedia Movie Data Set](https://github.com/prust/wikipedia-movie-data "Wikipedia Movie Data Set" ) to populate a DynamoDB table called "films". When running locally, the API uses DynamoDB local to simulate data storage. I am new to rust, but decided to use this opportunity to try and learn and improve my skills. 

This is not complete yet. Please see the TODOS at the bottom of this README. 

# Running the code
To run the code locally, you will need rust installed locally and `docker` installed to run dynamodb-local. Checkout this repository and from the root run `cargo run`. The development server will run on `localhost:3030`.  You can modify these parameters in the `src/main.rs` module. 

# Architecture Diagram
NOTE that this is an aspirational architecture diagram at this stage! The API is not currently deployed in AWS. 

![An architecture diagram](./arch.png?raw=true "Architecture")

# Architecture Notes
The API can be run in Kubernetes (k8s) by first building, pushing, and tagging a container with the API. You can package the API using the Dockerfile provided.  To run it on a k8s cluster, you will need to create a deployment and a service corresponding to that deployment. You will also need to create a `serviceAccount` for the API to be able to get temporary AWS credentials. Using the `serviceAccount` the API can obtain credentials to be able to read from S3 and write to DynamoDB. 

# Code Notes
The code uses the [Warp Web Framework](https://docs.rs/warp/latest/warp/ "Warp web framework") to implement the API. It also makes use of the [AWS SDK for Rust](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/rust_dynamodb_code_examples.html "AWS Rust SDK") to interact with AWS. The `src/models.rs` directory contains the data model for film and helper traits to convert it to formats that DynamoDB expects. The `src/handlers.rs` module provides HTTP handlers for the API and the `src/filters.rs` module provides path-based routing, to extract path and query fragments. 

The "movie dataset" is currently a large json file called `src/ddb/t.json` and for testing this is loaded and `batch_uploaded()` to DynamoDB.  In production, this would point to an S3 bucket instead. 


# Data Storage
DynamoDB was chosen as the data storage solution for a few reasons:
 * The source of the film data will be available in Amazon S3 and having a managed solution for data, like Amazon DynamoDB, lowers the operational complexity of the solution
 * Using a schemaless data-store allows us flexibility to update the schema to add additional metadata about films. Today, we require only the title and year as mandatory attributes, the schema can be extended in the future to say, incorporate IMDB ratings for films without much difficulty. 
 * In the future if we decide to run this API in a multi-region setup, we can use DynamoDB Global tables to provide globally available data storage
 * DynamoDB supports encryption at rest either using an AWS KMS key or a customer managed KMS key 


# Encryption in transit 
Currently the API listens on an HTTP port. We can add encryption in transit in a few ways:
 * Using a loadbalancer resource to terminate TLS. In AWS we could use an Application Load Balancer (ALB)
 * Running a sidecar container with envoy or similar tools to provide mutual TLS authentication for clients.
 * Implementing End-to-end TLS by running a TLS listener in the API instead of a plain-HTTP listener. 


# TODO
Due to a lack of time, I have not been able to implement and test these features: 
  * CRUD features
  * Search by attributes other than film year.
  * End-to-end testing in AWS, I have run this locally 
  * AWS Role policy creation for the API to be able to fetch objects from S3 and read/write to a DynamoDB table
  * Implement & run statistical benchmarking to figure out P99 latency
