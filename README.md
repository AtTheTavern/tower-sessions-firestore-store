<h1 align="center">
    tower-sessions-firestore-store
</h1>

<p align="center">
    tower-sessions store using Cloud Firestore
</p>

<div align="center">
    <a href="https://crates.io/crates/tower-sessions-firestore-store">
        <img src="https://img.shields.io/crates/v/tower-sessions-firestore-store.svg" />
    </a>
    <a href="https://docs.rs/tower-sessions-firestore-store">
        <img src="https://docs.rs/tower-sessions-firestore-store/badge.svg" />
    </a>
</div>

## Overview

This crate provides a [Cloud Firestore][firestore] store for [tower-sessions][tower-sessions].

## 📦 Install

To use the crate in your project, add the following to your `Cargo.toml` file:

```toml
[dependencies]
firestore = "0.39"
tower-sessions = "0.10"
tower-sessions-firestore-store = "0.2"
```

## Usage

Set up a Firestore database and configure the store with this database and a collection name. Can be tested locally with the [Firestore Emulator][firestore-emulator] or [Firebase Emulator UI][firebase-ui].

### `axum` Example

```rs
use std::net::SocketAddr;

use axum::{response::IntoResponse, routing::get, Router};
use firestore::FirestoreDb;
use serde::{Deserialize, Serialize};
use time::Duration;
use tower_sessions::{Expiry, Session, SessionManagerLayer};
use tower_sessions_firestore_store::FirestoreStore;

const COUNTER_KEY: &str = "counter";

#[derive(Default, Deserialize, Serialize)]
struct Counter(usize);

#[tokio::main]
async fn main() {
    let google_cloud_project = std::option_env!("GOOGLE_CLOUD_PROJECT")
        .expect("Missing GOOGLE_CLOUD_PROJECT.")
        .to_string();
    let db = FirestoreDb::new(google_cloud_project)
        .await
        .expect("Could not create FirestoreDb.");

    let session_store = FirestoreStore::new(db, "tower-sessions".to_string());
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(10)));

    let app = Router::new().route("/", get(handler)).layer(session_layer);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn handler(session: Session) -> impl IntoResponse {
    let counter: Counter = session.get(COUNTER_KEY).await.unwrap().unwrap_or_default();
    session.insert(COUNTER_KEY, counter.0 + 1).await.unwrap();
    format!("Current count: {}", counter.0)
}
```

You can find this [example][counter-example] in the [example directory][examples].

[counter-example]: https://github.com/AtTheTavern/tower-sessions-firestore-store/tree/main/examples/firestore-store.rs
[examples]: https://github.com/AtTheTavern/tower-sessions-firestore-store/tree/main/examples
[firebase-ui]: https://firebaseopensource.com/projects/firebase/firebase-tools-ui/
[firestore]: https://cloud.google.com/firestore
[firestore-emulator]: https://firebase.google.com/docs/emulator-suite/connect_firestore
[tower-sessions]: https://crates.io/crates/tower-sessions
