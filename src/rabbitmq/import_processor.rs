use anyhow::{anyhow, Context};
use chrono::NaiveDateTime;
use serde::Deserialize;
use sqlx::MySqlPool;
use tracing::{info, warn};

// ── Message schema ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ImportMessage {
    pub import_id: String,
    pub importer: String,
    pub transactions: Vec<ImportTransaction>,
}

#[derive(Debug, Deserialize)]
pub struct ImportTransaction {
    pub account_code: String,
    pub datetime: String,
    pub amount: f64,
    pub description: String,
    pub transaction_type_code: String,
    pub fingerprint: String,
}

// ── Public entry point ─────────────────────────────────────────────────────────

/// Deserialise a raw message payload and insert transactions into the database.
///
/// Returns the number of newly inserted rows (duplicates are silently skipped).
pub async fn process_import_message(
    pool: &MySqlPool,
    payload: &str,
) -> Result<usize, anyhow::Error> {
    let msg: ImportMessage =
        serde_json::from_str(payload).context("Failed to deserialise import message")?;

    info!(
        "Processing import batch {} from importer '{}' ({} transactions)",
        msg.import_id,
        msg.importer,
        msg.transactions.len()
    );

    let mut inserted = 0usize;

    for t in &msg.transactions {
        match insert_transaction(pool, t).await {
            Ok(true) => inserted += 1,
            Ok(false) => {
                info!(
                    "Skipped duplicate transaction (fingerprint={})",
                    t.fingerprint
                );
            }
            Err(e) => {
                // Log and continue — one bad row should not abort the whole batch
                warn!(
                    "Failed to insert transaction (fingerprint={}): {}",
                    t.fingerprint, e
                );
            }
        }
    }

    Ok(inserted)
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Insert a single transaction. Returns `true` if a new row was inserted,
/// `false` if the fingerprint already existed (duplicate).
async fn insert_transaction(
    pool: &MySqlPool,
    t: &ImportTransaction,
) -> Result<bool, anyhow::Error> {
    let account_id = resolve_account_id(pool, &t.account_code).await?;
    let transaction_type_id = resolve_transaction_type_id(pool, &t.transaction_type_code).await?;

    let datetime = NaiveDateTime::parse_from_str(&t.datetime, "%Y-%m-%d %H:%M:%S")
        .with_context(|| format!("Invalid datetime: {}", t.datetime))?;

    let result = sqlx::query(
        r#"
        INSERT IGNORE INTO transactions
            (account_id, transaction_type_id, datetime, amount, description, note, fingerprint)
        VALUES (?, ?, ?, ?, ?, NULL, ?)
        "#,
    )
    .bind(account_id)
    .bind(transaction_type_id)
    .bind(datetime)
    .bind(t.amount)
    .bind(&t.description)
    .bind(&t.fingerprint)
    .execute(pool)
    .await
    .context("Failed to execute INSERT IGNORE")?;

    Ok(result.rows_affected() > 0)
}

async fn resolve_account_id(pool: &MySqlPool, code: &str) -> Result<i64, anyhow::Error> {
    let row = sqlx::query_scalar::<_, i64>("SELECT id FROM accounts WHERE code = ?")
        .bind(code)
        .fetch_optional(pool)
        .await
        .context("Failed to query accounts")?;

    row.ok_or_else(|| anyhow!("Account with code '{}' not found", code))
}

async fn resolve_transaction_type_id(pool: &MySqlPool, code: &str) -> Result<i64, anyhow::Error> {
    let row = sqlx::query_scalar::<_, i64>("SELECT id FROM transaction_types WHERE code = ?")
        .bind(code)
        .fetch_optional(pool)
        .await
        .context("Failed to query transaction_types")?;

    row.ok_or_else(|| anyhow!("Transaction type with code '{}' not found", code))
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialise_valid_message() {
        let json = r#"{
            "import_id": "abc-123",
            "importer": "pcfinancial",
            "transactions": [
                {
                    "account_code": "PCFINANCIAL",
                    "datetime": "2024-01-15 12:00:00",
                    "amount": -50.00,
                    "description": "Coffee",
                    "transaction_type_code": "PURCHASE",
                    "fingerprint": "d41d8cd98f00b204e9800998ecf8427e"
                }
            ]
        }"#;

        let msg: ImportMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.import_id, "abc-123");
        assert_eq!(msg.importer, "pcfinancial");
        assert_eq!(msg.transactions.len(), 1);

        let t = &msg.transactions[0];
        assert_eq!(t.account_code, "PCFINANCIAL");
        assert_eq!(t.transaction_type_code, "PURCHASE");
        assert_eq!(t.amount, -50.00);
    }

    #[test]
    fn test_deserialise_invalid_message_returns_error() {
        let json = r#"{"not": "valid"}"#;
        // Missing required fields → serde_json should fail
        let result: Result<ImportMessage, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialise_empty_transactions() {
        let json = r#"{
            "import_id": "empty-batch",
            "importer": "nu",
            "transactions": []
        }"#;
        let msg: ImportMessage = serde_json::from_str(json).unwrap();
        assert!(msg.transactions.is_empty());
    }
}
