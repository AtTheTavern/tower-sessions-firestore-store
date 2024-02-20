use std::{collections::HashMap, str::FromStr, sync::Arc};

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use firestore::FirestoreDb;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use tower_sessions_core::{
    session::{Id, Record},
    session_store, SessionStore,
};

/// An error type for `FirestoreStore`.
#[derive(thiserror::Error, Debug)]
pub enum FirestoreStoreError {
    /// A variant to map to `firestore::errors::FirestoreError` errors.
    #[error(transparent)]
    Firestore(#[from] firestore::errors::FirestoreError),
}

impl From<FirestoreStoreError> for session_store::Error {
    fn from(err: FirestoreStoreError) -> Self {
        match err {
            FirestoreStoreError::Firestore(inner) => {
                session_store::Error::Backend(inner.to_string())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct FirestoreStore {
    pub db: Arc<FirestoreDb>,
    pub collection_id: String,
}

impl FirestoreStore {
    pub fn new(db: FirestoreDb, collection_id: String) -> Self {
        Self {
            db: Arc::new(db),
            collection_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FirestoreRecord {
    pub id: String,
    pub data: HashMap<String, Value>,
    pub expiry_date: OffsetDateTime,
}

impl From<Record> for FirestoreRecord {
    fn from(record: Record) -> Self {
        Self {
            id: record.id.to_string(),
            data: record.data,
            expiry_date: record.expiry_date,
        }
    }
}

impl From<FirestoreRecord> for Record {
    fn from(record: FirestoreRecord) -> Self {
        Self {
            id: Id::from_str(&record.id).unwrap_or_default(),
            data: record.data,
            expiry_date: record.expiry_date,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FirestoreDocument {
    record: FirestoreRecord,
    #[serde(
        rename = "expireAt",
        with = "firestore::serialize_as_optional_timestamp"
    )]
    expire_at: Option<DateTime<Utc>>,
}

impl From<Record> for FirestoreDocument {
    fn from(record: Record) -> Self {
        let expire_at = match Utc.timestamp_opt(
            record.expiry_date.unix_timestamp(),
            record.expiry_date.nanosecond(),
        ) {
            chrono::offset::LocalResult::Single(expire_at) => Some(expire_at),
            _ => None,
        };
        Self {
            record: record.into(),
            expire_at,
        }
    }
}

#[async_trait]
impl SessionStore for FirestoreStore {
    async fn save(&self, record: &Record) -> session_store::Result<()> {
        let doc = FirestoreDocument::from(record.clone());
        self.db
            .fluent()
            .update() // Update will create the document if it doesn't exist
            .in_col(self.collection_id.as_ref())
            .document_id(&record.id.to_string())
            .object(&doc)
            .execute()
            .await
            .map_err(FirestoreStoreError::Firestore)?;
        Ok(())
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let doc: Option<FirestoreDocument> = self
            .db
            .fluent()
            .select()
            .by_id_in(self.collection_id.as_ref())
            .obj()
            .one(session_id.to_string())
            .await
            .map_err(FirestoreStoreError::Firestore)?;

        if let Some(doc) = doc {
            Ok(Some(doc.record.into()))
        } else {
            Ok(None)
        }
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        self.db
            .fluent()
            .delete()
            .from(self.collection_id.as_ref())
            .document_id(session_id.to_string())
            .execute()
            .await
            .map_err(FirestoreStoreError::Firestore)?;
        Ok(())
    }
}
