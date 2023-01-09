# Rust Web Example

An example web application using Rust for the backend.

## Environment Variables

### Required Environment Variables
- `TIBERIUS_USERNAME`: the username Tiberius will log in as to the MSSQL server
- `TIBERIUS_PASSWORD`: the password Tiberius will use to log in to the MSSQL server

### Optional Environment Variables
- `TIBERIUS_HOST`: the database host Tiberius should connect to (127.0.0.1)
- `TIBERIUS_PORT`: the database port Tiberius should connect to (1433)

## Running the App

`cargo run`

## Testing the App

`cargo test`
`cargo clippy`

## Available routes

- GET `/api/v1/hospitals`
- GET `/api/v1/hospitals/{name}`
- POST `/api/v1/hospitals/{name}` Patient
- DELETE `/api/v1/hospitals/{name}/{patient_id}`

## Helpful links

- [Actix](https://actix.rs/docs/getting-started)
- [Mockall](https://crates.io/crates/mockall)
- [Tiberius](https://crates.io/crates/tiberius)