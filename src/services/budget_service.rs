use crate::models::budgets::{BudgetMonthTotal, BudgetRecord};
use chrono::{Days, Local, Months, NaiveDate};
use sqlx::{FromRow, MySqlPool, Row};

#[derive(Debug, Clone)]
pub struct BudgetMutationPayload {
    pub account_id: i64,
    pub date: NaiveDate,
    pub amount: f64,
    pub description: String,
    pub processed: bool,
    pub note: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BudgetGenerationPayload {
    pub end_date: NaiveDate,
    pub generate_only_for_future: bool,
}

#[derive(Debug, Clone, FromRow)]
struct BudgetSetupRecord {
    id: i64,
    date: NaiveDate,
    is_repeatle: bool,
    repeat_frequency: Option<String>,
    end_date: Option<NaiveDate>,
    description: String,
    amount: f64,
}

#[derive(Debug, Clone)]
struct BudgetSetupMetadata {
    budget_setup_id: i64,
    is_repeatle: bool,
}

fn next_budget_date(date: NaiveDate, frequency: &str) -> Option<NaiveDate> {
    match frequency {
        "WEEKLY" => date.checked_add_days(Days::new(7)),
        "BIWEEKLY" => date.checked_add_days(Days::new(14)),
        "MONTHLY" => date.checked_add_months(Months::new(1)),
        "YEARLY" => date.checked_add_months(Months::new(12)),
        _ => None,
    }
}

fn build_budget_dates(
    start_date: NaiveDate,
    is_repeatle: bool,
    repeat_frequency: Option<&str>,
    setup_end_date: Option<NaiveDate>,
    generation_end_date: NaiveDate,
) -> Vec<NaiveDate> {
    let effective_end_date = setup_end_date
        .map(|value| value.min(generation_end_date))
        .unwrap_or(generation_end_date);

    if start_date > effective_end_date {
        return Vec::new();
    }

    if !is_repeatle {
        return vec![start_date];
    }

    let Some(frequency) = repeat_frequency else {
        return vec![start_date];
    };

    let mut result = Vec::new();
    let mut current_date = start_date;

    while current_date <= effective_end_date {
        result.push(current_date);

        let Some(next_date) = next_budget_date(current_date, frequency) else {
            break;
        };

        current_date = next_date;
    }

    result
}

fn filter_budget_dates_for_generation(
    budget_dates: Vec<NaiveDate>,
    generate_only_for_future: bool,
    current_date: NaiveDate,
) -> Vec<NaiveDate> {
    if !generate_only_for_future {
        return budget_dates;
    }

    budget_dates
        .into_iter()
        .filter(|budget_date| *budget_date >= current_date)
        .collect()
}

pub async fn list_budget_month_totals(
    pool: &MySqlPool,
    page: u32,
    page_size: u32,
) -> Result<(Vec<BudgetMonthTotal>, u64), anyhow::Error> {
    let total_row = sqlx::query(
        r#"
        SELECT COUNT(*) AS total_count
        FROM (
            SELECT YEAR(b.date) AS year, MONTH(b.date) AS month
            FROM budget b
            GROUP BY YEAR(b.date), MONTH(b.date)
        ) budget_months
        "#,
    )
    .fetch_one(pool)
    .await?;

    let total_count = total_row.try_get::<i64, _>("total_count")? as u64;
    let offset = ((page - 1) * page_size) as i64;

    let totals = sqlx::query_as::<_, BudgetMonthTotal>(
        r#"
        SELECT
            CAST(YEAR(b.date) AS SIGNED) AS year,
            CAST(MONTH(b.date) AS SIGNED) AS month,
            CONCAT(YEAR(b.date), '-', LPAD(MONTH(b.date), 2, '0')) AS month_label,
            SUM(b.amount) AS total_amount
        FROM budget b
        GROUP BY year, month, month_label 
        ORDER BY year ASC, month ASC
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(page_size as i64)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok((totals, total_count))
}

pub async fn list_budget_details(
    pool: &MySqlPool,
    year: i32,
    month: u32,
) -> Result<Vec<BudgetRecord>, anyhow::Error> {
    let budgets = sqlx::query_as::<_, BudgetRecord>(
        r#"
        SELECT
            b.id,
            b.budget_setup_id,
            bs.account_id,
            a.code AS account_code,
            a.name AS account_name,
            b.date,
            b.amount,
            b.description,
            b.processed,
            bs.note,
            bs.is_repeatle,
            bs.repeat_frequency
        FROM budget b
        INNER JOIN budget_setup bs ON bs.id = b.budget_setup_id
        INNER JOIN accounts a ON a.id = bs.account_id
        WHERE YEAR(b.date) = ?
        AND MONTH(b.date) = ?
        ORDER BY b.date ASC, b.amount ASC, b.id ASC
        "#,
    )
    .bind(year)
    .bind(month)
    .fetch_all(pool)
    .await?;

    Ok(budgets)
}

pub async fn get_budget_by_id(
    pool: &MySqlPool,
    budget_id: i64,
) -> Result<Option<BudgetRecord>, anyhow::Error> {
    let budget = sqlx::query_as::<_, BudgetRecord>(
        r#"
        SELECT
            b.id,
            b.budget_setup_id,
            bs.account_id,
            a.code AS account_code,
            a.name AS account_name,
            b.date,
            b.amount,
            b.description,
            b.processed,
            bs.note,
            bs.is_repeatle,
            bs.repeat_frequency
        FROM budget b
        INNER JOIN budget_setup bs ON bs.id = b.budget_setup_id
        INNER JOIN accounts a ON a.id = bs.account_id
        WHERE b.id = ?
        "#,
    )
    .bind(budget_id)
    .fetch_optional(pool)
    .await?;

    Ok(budget)
}

async fn get_budget_setup_metadata(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    budget_id: i64,
) -> Result<Option<BudgetSetupMetadata>, anyhow::Error> {
    let metadata = sqlx::query(
        r#"
        SELECT b.budget_setup_id, bs.is_repeatle
        FROM budget b
        INNER JOIN budget_setup bs ON bs.id = b.budget_setup_id
        WHERE b.id = ?
        "#,
    )
    .bind(budget_id)
    .fetch_optional(&mut **tx)
    .await?;

    Ok(metadata.map(|row| BudgetSetupMetadata {
        budget_setup_id: row.try_get::<i64, _>("budget_setup_id").unwrap_or_default(),
        is_repeatle: row.try_get::<bool, _>("is_repeatle").unwrap_or(false),
    }))
}

pub async fn create_budget(
    pool: &MySqlPool,
    payload: BudgetMutationPayload,
) -> Result<BudgetRecord, anyhow::Error> {
    let mut tx = pool.begin().await?;

    let setup_result = sqlx::query(
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
    .bind(false)
    .bind(Option::<String>::None)
    .bind(Option::<NaiveDate>::None)
    .bind(&payload.description)
    .bind(payload.amount)
    .bind(payload.note.clone())
    .execute(&mut *tx)
    .await?;

    let budget_setup_id = setup_result.last_insert_id() as i64;

    let budget_result = sqlx::query(
        r#"
        INSERT INTO budget (budget_setup_id, date, amount, description, processed)
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(budget_setup_id)
    .bind(payload.date)
    .bind(payload.amount)
    .bind(payload.description)
    .bind(payload.processed)
    .execute(&mut *tx)
    .await?;

    let budget_id = budget_result.last_insert_id() as i64;

    tx.commit().await?;

    get_budget_by_id(pool, budget_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to load created budget"))
}

pub async fn update_budget(
    pool: &MySqlPool,
    budget_id: i64,
    payload: BudgetMutationPayload,
) -> Result<BudgetRecord, anyhow::Error> {
    let mut tx = pool.begin().await?;
    let Some(metadata) = get_budget_setup_metadata(&mut tx, budget_id).await? else {
        tx.rollback().await?;
        return Err(anyhow::anyhow!("Budget not found"));
    };

    let result = sqlx::query(
        r#"
        UPDATE budget
        SET date = ?, amount = ?, description = ?, processed = ?
        WHERE id = ?
        "#,
    )
    .bind(payload.date)
    .bind(payload.amount)
    .bind(&payload.description)
    .bind(payload.processed)
    .bind(budget_id)
    .execute(&mut *tx)
    .await?;

    if result.rows_affected() == 0 {
        tx.rollback().await?;
        return Err(anyhow::anyhow!("Budget not found"));
    }

    if !metadata.is_repeatle {
        sqlx::query(
            r#"
            UPDATE budget_setup
            SET account_id = ?, date = ?, description = ?, amount = ?, note = ?
            WHERE id = ?
            "#,
        )
        .bind(payload.account_id)
        .bind(payload.date)
        .bind(payload.description)
        .bind(payload.amount)
        .bind(payload.note)
        .bind(metadata.budget_setup_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    get_budget_by_id(pool, budget_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Budget not found"))
}

pub async fn delete_budget(pool: &MySqlPool, budget_id: i64) -> Result<(), anyhow::Error> {
    let mut tx = pool.begin().await?;
    let Some(metadata) = get_budget_setup_metadata(&mut tx, budget_id).await? else {
        tx.rollback().await?;
        return Err(anyhow::anyhow!("Budget not found"));
    };

    let result = sqlx::query("DELETE FROM budget WHERE id = ?")
        .bind(budget_id)
        .execute(&mut *tx)
        .await?;

    if result.rows_affected() == 0 {
        tx.rollback().await?;
        return Err(anyhow::anyhow!("Budget not found"));
    }

    if !metadata.is_repeatle {
        let remaining_row =
            sqlx::query("SELECT COUNT(*) AS total_count FROM budget WHERE budget_setup_id = ?")
                .bind(metadata.budget_setup_id)
                .fetch_one(&mut *tx)
                .await?;
        let remaining_count = remaining_row.try_get::<i64, _>("total_count")?;

        if remaining_count == 0 {
            sqlx::query("DELETE FROM budget_setup WHERE id = ?")
                .bind(metadata.budget_setup_id)
                .execute(&mut *tx)
                .await?;
        }
    }

    tx.commit().await?;
    Ok(())
}

pub async fn generate_budgets(
    pool: &MySqlPool,
    payload: BudgetGenerationPayload,
) -> Result<u64, anyhow::Error> {
    let current_date = Local::now().date_naive();
    let setups = sqlx::query_as::<_, BudgetSetupRecord>(
        r#"
        SELECT
            id,
            date,
            is_repeatle,
            repeat_frequency,
            end_date,
            description,
            amount
        FROM budget_setup
        WHERE date <= ?
        ORDER BY date ASC, id ASC
        "#,
    )
    .bind(payload.end_date)
    .fetch_all(pool)
    .await?;

    let mut tx = pool.begin().await?;
    let mut created_count = 0_u64;

    for setup in setups {
        let budget_dates = build_budget_dates(
            setup.date,
            setup.is_repeatle,
            setup.repeat_frequency.as_deref(),
            setup.end_date,
            payload.end_date,
        );
        let budget_dates = filter_budget_dates_for_generation(
            budget_dates,
            payload.generate_only_for_future,
            current_date,
        );

        for budget_date in budget_dates {
            let result = sqlx::query(
                r#"
                INSERT INTO budget (budget_setup_id, date, amount, description, processed)
                SELECT ?, ?, ?, ?, ?
                FROM DUAL
                WHERE NOT EXISTS (
                    SELECT 1
                    FROM budget existing_budget
                    WHERE existing_budget.budget_setup_id = ?
                    AND existing_budget.date = ?
                )
                "#,
            )
            .bind(setup.id)
            .bind(budget_date)
            .bind(setup.amount)
            .bind(&setup.description)
            .bind(false)
            .bind(setup.id)
            .bind(budget_date)
            .execute(&mut *tx)
            .await?;

            created_count += result.rows_affected();
        }
    }

    tx.commit().await?;

    Ok(created_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_budget_dates_for_non_repeatable_setup() {
        let result = build_budget_dates(
            NaiveDate::from_ymd_opt(2026, 4, 16).unwrap(),
            false,
            None,
            None,
            NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        );

        assert_eq!(result, vec![NaiveDate::from_ymd_opt(2026, 4, 16).unwrap()]);
    }

    #[test]
    fn test_build_budget_dates_for_monthly_setup() {
        let result = build_budget_dates(
            NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            true,
            Some("MONTHLY"),
            None,
            NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(),
        );

        assert_eq!(
            result,
            vec![
                NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
                NaiveDate::from_ymd_opt(2026, 2, 28).unwrap(),
                NaiveDate::from_ymd_opt(2026, 3, 28).unwrap(),
                NaiveDate::from_ymd_opt(2026, 4, 28).unwrap(),
            ]
        );
    }

    #[test]
    fn test_build_budget_dates_for_biweekly_setup_with_end_date() {
        let result = build_budget_dates(
            NaiveDate::from_ymd_opt(2026, 4, 10).unwrap(),
            true,
            Some("BIWEEKLY"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 8).unwrap()),
            NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        );

        assert_eq!(
            result,
            vec![
                NaiveDate::from_ymd_opt(2026, 4, 10).unwrap(),
                NaiveDate::from_ymd_opt(2026, 4, 24).unwrap(),
                NaiveDate::from_ymd_opt(2026, 5, 8).unwrap(),
            ]
        );
    }

    #[test]
    fn test_filter_budget_dates_for_generation_with_future_only() {
        let result = filter_budget_dates_for_generation(
            vec![
                NaiveDate::from_ymd_opt(2026, 4, 9).unwrap(),
                NaiveDate::from_ymd_opt(2026, 4, 10).unwrap(),
                NaiveDate::from_ymd_opt(2026, 4, 11).unwrap(),
            ],
            true,
            NaiveDate::from_ymd_opt(2026, 4, 10).unwrap(),
        );

        assert_eq!(
            result,
            vec![
                NaiveDate::from_ymd_opt(2026, 4, 10).unwrap(),
                NaiveDate::from_ymd_opt(2026, 4, 11).unwrap(),
            ]
        );
    }

    #[test]
    fn test_filter_budget_dates_for_generation_without_future_only() {
        let dates = vec![
            NaiveDate::from_ymd_opt(2026, 4, 9).unwrap(),
            NaiveDate::from_ymd_opt(2026, 4, 10).unwrap(),
        ];

        let result = filter_budget_dates_for_generation(
            dates.clone(),
            false,
            NaiveDate::from_ymd_opt(2026, 4, 10).unwrap(),
        );

        assert_eq!(result, dates);
    }
}
