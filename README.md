# Rust Web Example
An example web application using Rust for the backend. It consists of 3 parts:
1. `admission`: a REST API that allows authenticated users to admit patients to
   a hospital, unadmit patients from a hospital, and view the patients admitted
   to a hospital.
2. `census`: a simple script which retrieves data from the `admission` API and
   produces a report on how many patients are admitted.
3. `common`: contains code shared by both other parts.
4. `complement`: computes set complement for `admission`

## Setting up the database
1. create a database `RustDB`
2. within the `RustDB` database, create a schema `rust`
3. create a new user account for Tiberius, and map it to `RustDB` with `rust` as
   its default schema.
4. give the Tiberius account permission to create table, delete, insert, select,
   and update.
5. run the app with `cargo run -p admission -- --setup`

## Setting up OpenID
This demo uses OpenID to authenticate users with an external service. I tested 
it with [https://auth0.com/](Auth0), but other identity providers should work.
Regardless of which provider you choose, you'll need their URL, as well as a
client-ID and -secret. Be sure to add `http://localhost:8080/openid` as a 
callback URL!

## Required Environment Variables
- `JWT_SECRET`: the secret key to use for signing JSON web tokens
- `TIBERIUS_USERNAME`: the username Tiberius will log in as to the MSSQL server
- `TIBERIUS_PASSWORD`: the password Tiberius will use to log in to the MSSQL server
- `OPENID_URL`: the URL for the OpenID provider to use. Formatted as `https://example.com/`
- `OPENID_CLIENT_ID`: the app's client ID registered with the OpenID provider
- `OPENID_CLIENT_SECRET`: the app's secret registered with the OpenID provider

## Running the App
`cargo run -p admission`
and in another terminal
`cargo run -p complement`

## Testing the App
`cargo test`
`cargo clippy`

## Demo

This app demonstrates many of the basic features common to most REST APIs.
1. run the app and open `Postman`
2. make a `GET` request to `localhost:8080/api/v1/hospitals` - should receive `401 unauthorized`
3. go to `localhost:8080/login` in your browser and sign in with an account. Notice the JWT field
   OR
   `POST` to `localhost:8080/jwt` with the following raw JSON body:
   ```
   {
      "email": "example@dsh.ca.gov"
   }
   ```
   you should receive a long string with 2 '.'s in it
4. in your Headers, set `Authorization` to `Bearer ` (with a space), followed by
   the response from your previous request.
5. `GET localhost:8080/api/v1/hospitals` again - you should receive a `200` response
6. you can check out `GET localhost:8080/api/v1/hospitals/napa` or other hospitals
7. add a patient to the waitlist by `POST`ing to `localhost:8080/api/v1/waitlist`
   with the following raw JSON body:
   ```
   {
       "name": "John Brown"
   }
   ```
   You should receive `401 unauthorized`.
8. `POST` to `localhost:8080/jwt`, but this time as
   ```
   {
      "email": "admin@dsh.ca.gov" 
   }
   ```
9. repeat step 7 after setting your new bearer token
10. `GET localhost:8080/api/v1/waitlist`
11. `POST localhost:8080/api/v1/hospitals/admit-from-waitlist` - notice
   which hospital John Brown was admitted to, as well as their ID
12. unadmit John Brown using `DELETE localhost:8080/api/v1/hospitals/{hospital}/{ID}`,
   where `ID` is John Brown's ID from the previous step. You should receive `204 No Content`.
13. `GET localhost:8080/api/v1/hospitals/{hospital}` to confirm John Brown has been
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
- and more!