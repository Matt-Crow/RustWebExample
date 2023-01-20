# Rust Web Example
An example web application using Rust for the backend.

## Setting up the database
1. create a database `RustDB`
2. within the `RustDB` database, create a schema `rust`
3. create a new user account for Tiberius, and map it to `RustDB` with `rust` as
   its default schema.
4. give the Tiberius account permission to create table, delete, insert, select,
   and update.
5. run the app with `cargo run -- --setup`

## Required Environment Variables
- `JWT_SECRET`: the secret key to use for signing JSON web tokens
- `TIBERIUS_USERNAME`: the username Tiberius will log in as to the MSSQL server
- `TIBERIUS_PASSWORD`: the password Tiberius will use to log in to the MSSQL server

## Running the App
`cargo run`

## Testing the App
`cargo test`
`cargo clippy`

## Demo
This app demonstrates many of the basic features common to most REST APIs.
1. run the app and open `Postman`
2. make a `GET` request to `localhost:8080/api/v1/hospitals` - should receive `401 unauthorized`
3. `POST` to `localhost:8080/jwt` with the following raw JSON body:
   ```
   {
       "email": "you can put anything between these two quotes"
   }
   ```
   you should receive a long string with 2 '.'s in it
4. in your Headers, set `Authorization` to `Bearer ` (with a space), followed by
   the response from your previous request.
5. `GET localhost:8080/api/v1/hospitals` again - you should receive a `200` response
6. you can check out `GET localhost:8080/api/v1/hospitals/napa` or other hospitals
7. admit a patient to Napa by `POST`ing to `localhost:8080/api/v1/hospitals/napa`
   with the following raw JSON body:
   ```
   {
       "name": "John Brown"
   }
   ```
8. `GET localhost:8080/api/v1/hospitals/napa`, then locate John Brown's ID
9. unadmit John Brown using `DELETE localhost:8080/api/v1/hospitals/napa/ID`,
   where `ID` is John Brown's ID from the previous step. You should receive `204 No Content`.
10. `GET localhost:8080/api/v1/hospitals/napa` to confirm John Brown has been
    unadmitted.

## Libraries used
- [Actix Web](https://actix.rs/) asynchronous web framework
- [Actix Web HTTPAuth](https://crates.io/crates/actix-web-httpauth) authentication
- [BB8](https://crates.io/crates/bb8) database connection pool
- [BB8 Tiberius](https://crates.io/crates/bb8-tiberius) BB8 implementation for Tiberius
- [Futures util](https://crates.io/crates/futures-util) asynchronous utilities
- [Chrono](https://crates.io/crates/chrono) datetime utilities
- [JSON Web Token](https://crates.io/crates/jsonwebtoken) JWT
- [Mockall](https://crates.io/crates/mockall/0.9.1) dependency mocker for testing
- [Reqwest](https://crates.io/crates/reqwest) HTTP request client
- [Serde](https://serde.rs/) JSON serialization / deserialization
- [Tiberius](https://crates.io/crates/tiberius) Microsoft SQL client
- [Tokio](https://tokio.rs/) asynchronous runtime