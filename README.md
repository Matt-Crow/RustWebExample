# Rust Web Example

An example web application using Rust for the backend.

## Running the App

`cargo run`

## Testing the App

`cargo test`
`cargo clippy`

## Available routes

- POST `localhost:8080/api/v1/anchors`
- GET `localhost:8080/api/v1/anchors`
- GET `localhost:8080/api/v1/anchors/{id}`
- DELETE `localhost:8080/api/v1/anchors/{id}`
- PUT `localhost:8080/api/v1/anchors/{id}`
- GET `localhost:8080/api/v1/forecast/{location}/{days}`
- GET `localhost:8080/api/v1/forecast/to-farenheight/{celsius}`
- GET `localhost:8080/api/v1/forecast/to-celsius/{farenheight}`

## Helpful links

- [Actix](https://actix.rs/docs/getting-started)
- [Mockall](https://crates.io/crates/mockall)
- [Tiberius](https://crates.io/crates/tiberius)