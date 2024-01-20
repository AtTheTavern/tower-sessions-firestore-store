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
