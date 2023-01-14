# Rust Web Example

An example web application using Rust for the backend.
TODO document how to set up DB.
Need to specify default schema of "rust" for Tiberius user.

## Environment Variables

### Required Environment Variables
- `JWT_SECRET`: the secret key to use for signing JSON web tokens
- `TIBERIUS_USERNAME`: the username Tiberius will log in as to the MSSQL server
- `TIBERIUS_PASSWORD`: the password Tiberius will use to log in to the MSSQL server
- `TIBERIUS_PIPE`: the named pipe Tiberius will use to connect to the MSSQL server

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