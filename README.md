# Rust Web Example

An example web application using Rust for the backend.
TODO document how to set up DB.
Need to specify default schema of "rust" for Tiberius user.

## Environment Variables

### Required Environment Variables
- `JWT_SECRET`: the secret key to use for signing JSON web tokens
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
- POST `/jwt` `{email: string}` to receive JWT

## Helpful links

- [Actix](https://actix.rs/docs/getting-started)
- [Mockall](https://crates.io/crates/mockall)
- [Tiberius](https://crates.io/crates/tiberius)