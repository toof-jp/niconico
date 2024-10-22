# niconico
![Crates.io Version](https://img.shields.io/crates/v/niconico?link=https%3A%2F%2Fcrates.io%2Fcrates%2Fniconico)
![docs.rs](https://img.shields.io/docsrs/niconico?link=https%3A%2F%2Fdocs.rs%2Fniconico%2Flatest%2Fniconico%2F)


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
