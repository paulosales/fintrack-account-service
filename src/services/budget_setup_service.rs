use crate::models::budget_setups::BudgetSetupRecord;
use anyhow::anyhow;
use chrono::NaiveDate;
use sqlx::{MySqlPool, Row};

const ALLOWED_REPEAT_FREQUENCIES: [&str; 4] = ["MONTHLY", "WEEKLY", "YEARLY", "BIWEEKLY"];

#[derive(Debug, Clone)]
pub struct BudgetSetupMutationPayload {
    pub account_id: i64,
    pub date: NaiveDate,
    pub is_repeatle: bool,
    pub repeat_frequency: Option<String>,
    pub end_date: Option<NaiveDate>,
    pub description: String,
    pub amount: f64,
    pub note: Option<String>,
}

fn normalize_repeat_configuration(
    payload: &BudgetSetupMutationPayload,
) -> Result<(Option<String>, Option<NaiveDate>), anyhow::Error> {
    if !payload.is_repeatle {
        return Ok((None, None));
    }

    let Some(repeat_frequency) = payload.repeat_frequency.clone() else {
        return Err(anyhow!("Repeat frequency is required for repeating budget setups"));
    };

    if !ALLOWED_REPEAT_FREQUENCIES.contains(&repeat_frequency.as_str()) {
        return Err(anyhow!("Invalid repeat frequency"));
    }

    if let Some(end_date) = payload.end_date {
        if end_date < payload.date {
            return Err(anyhow!("End date cannot be before the setup date"));
        }
    }

    Ok((Some(repeat_frequency), payload.end_date))
}

pub async fn list_budget_setups(
    pool: &MySqlPool,
    page: u32,
    page_size: u32,
) -> Result<(Vec<BudgetSetupRecord>, u64), anyhow::Error> {
    let total_row = sqlx::query("SELECT COUNT(*) AS total_count FROM budget_setup")
        .fetch_one(pool)
        .await?;
    let total_count = total_row.try_get::<i64, _>("total_count")? as u64;
    let offset = ((page - 1) * page_size) as i64;

    let setups = sqlx::query_as::<_, BudgetSetupRecord>(
        r#"
        SELECT
            bs.id,
            bs.account_id,
            a.code AS account_code,
            a.name AS account_name,
            bs.date,
            bs.is_repeatle,
            bs.repeat_frequency,
            bs.end_date,
            bs.description,
            bs.amount,
            bs.note
        FROM budget_setup bs
        INNER JOIN accounts a ON a.id = bs.account_id
        ORDER BY bs.date ASC, bs.id ASC
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(page_size as i64)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok((setups, total_count))
}

pub async fn get_budget_setup_by_id(
    pool: &MySqlPool,
    budget_setup_id: i64,
) -> Result<Option<BudgetSetupRecord>, anyhow::Error> {
    let setup = sqlx::query_as::<_, BudgetSetupRecord>(
        r#"
        SELECT
            bs.id,
            bs.account_id,
            a.code AS account_code,
            a.name AS account_name,
            bs.date,
            bs.is_repeatle,
            bs.repeat_frequency,
            bs.end_date,
            bs.description,
            bs.amount,
            bs.note
        FROM budget_setup bs
        INNER JOIN accounts a ON a.id = bs.account_id
        WHERE bs.id = ?
        "#,
    )
    .bind(budget_setup_id)
    .fetch_optional(pool)
    .await?;

    Ok(setup)
}

pub async fn create_budget_setup(
    pool: &MySqlPool,
    payload: BudgetSetupMutationPayload,
) -> Result<BudgetSetupRecord, anyhow::Error> {
    let (repeat_frequency, end_date) = normalize_repeat_configuration(&payload)?;

    let result = sqlx::query(
        r#"
        INSERT INTO budget_setup (
            account_id,
            date,
            is_repeatle,
            repeat_frequency,
            end_date,
            description,
            amount,
            note
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(payload.account_id)
    .bind(payload.date)
    .bind(payload.is_repeatle)
    .bind(repeat_frequency)
    .bind(end_date)
    .bind(payload.description)
    .bind(payload.amount)
    .bind(payload.note)
    .execute(pool)
    .await?;

    get_budget_setup_by_id(pool, result.last_insert_id() as i64)
        .await?
        .ok_or_else(|| anyhow!("Failed to load created budget setup"))
}

pub async fn update_budget_setup(
    pool: &MySqlPool,
    budget_setup_id: i64,
    payload: BudgetSetupMutationPayload,
) -> Result<BudgetSetupRecord, anyhow::Error> {
    let (repeat_frequency, end_date) = normalize_repeat_configuration(&payload)?;

    let result = sqlx::query(
        r#"
        UPDATE budget_setup
        SET
            account_id = ?,
            date = ?,
            is_repeatle = ?,
            repeat_frequency = ?,
            end_date = ?,
            description = ?,
            amount = ?,
            note = ?
        WHERE id = ?
        "#,
    )
    .bind(payload.account_id)
    .bind(payload.date)
    .bind(payload.is_repeatle)
    .bind(repeat_frequency)
    .bind(end_date)
    .bind(payload.description)
    .bind(payload.amount)
    .bind(payload.note)
    .bind(budget_setup_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(anyhow!("Budget setup not found"));
    }

    get_budget_setup_by_id(pool, budget_setup_id)
        .await?
        .ok_or_else(|| anyhow!("Budget setup not found"))
}

pub async fn delete_budget_setup(
    pool: &MySqlPool,
    budget_setup_id: i64,
) -> Result<(), anyhow::Error> {
    let result = sqlx::query("DELETE FROM budget_setup WHERE id = ?")
        .bind(budget_setup_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(anyhow!("Budget setup not found"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_payload() -> BudgetSetupMutationPayload {
        BudgetSetupMutationPayload {
            account_id: 1,
            date: NaiveDate::from_ymd_opt(2026, 4, 16).unwrap(),
            is_repeatle: true,
            repeat_frequency: Some("MONTHLY".to_string()),
            end_date: Some(NaiveDate::from_ymd_opt(2026, 12, 16).unwrap()),
            description: "Insurance".to_string(),
            amount: -123.45,
            note: Some("Monthly".to_string()),
        }
    }

    #[test]
    fn test_normalize_repeat_configuration_for_non_repeating_setup() {
        let mut payload = build_payload();
        payload.is_repeatle = false;

        let result = normalize_repeat_configuration(&payload).unwrap();

        assert_eq!(result, (None, None));
    }

    #[test]
    fn test_normalize_repeat_configuration_requires_frequency() {
        let mut payload = build_payload();
        payload.repeat_frequency = None;

        let result = normalize_repeat_configuration(&payload);

        assert!(result.is_err());
    }

    #[test]
    fn test_normalize_repeat_configuration_rejects_invalid_end_date() {
        let mut payload = build_payload();
        payload.end_date = Some(NaiveDate::from_ymd_opt(2026, 4, 10).unwrap());

        let result = normalize_repeat_configuration(&payload);

        assert!(result.is_err());
    }
}