# niconico
![Crates.io Version](https://img.shields.io/crates/v/niconico)
![docs.rs](https://img.shields.io/docsrs/niconico)

A Rust client library for Niconico authentication
```rust
use niconico::{login, Credentials};
use secrecy::ExposeSecret;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let credentials = envy::from_env::<Credentials>().unwrap();

    let user_session = login(credentials).await.unwrap();

    println!("{:?}", user_session.user_session.expose_secret());
}
```
