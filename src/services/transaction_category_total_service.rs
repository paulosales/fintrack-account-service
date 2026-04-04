use crate::models::transaction_category_totals::{
    TransactionCategoryTotal, TransactionCategoryTotalDetail,
};
use sqlx::MySqlPool;
use sqlx::Row;

pub async fn list_transaction_category_totals(
    pool: &MySqlPool,
    month: Option<u32>,
    year: Option<i32>,
    category_id: Option<i64>,
    page: u32,
    page_size: u32,
) -> Result<(Vec<TransactionCategoryTotal>, u64), anyhow::Error> {
    let total_row = sqlx::query(
        r#"
        SELECT COUNT(*) AS total_count
        FROM (
            SELECT
                CAST(YEAR(t.datetime) AS SIGNED) AS year,
                CAST(MONTH(t.datetime) AS SIGNED) AS month,
                cat.id AS category_id,
                cat.name AS category
            FROM transactions t
            INNER JOIN transactions_categories tc ON tc.transaction_id = t.id
            INNER JOIN categories cat ON tc.category_id = cat.id
            WHERE (? IS NULL OR MONTH(t.datetime) = ?)
            AND (? IS NULL OR YEAR(t.datetime) = ?)
            AND (? IS NULL OR cat.id = ?)
            GROUP BY 1, 2, 3, 4

            UNION

            SELECT
                CAST(YEAR(t.datetime) AS SIGNED) AS year,
                CAST(MONTH(t.datetime) AS SIGNED) AS month,
                cat.id AS category_id,
                cat.name AS category
            FROM sub_transactions st
            INNER JOIN sub_transactions_categories stc ON stc.sub_transaction_id = st.id
            INNER JOIN categories cat ON stc.category_id = cat.id
            INNER JOIN transactions t ON st.transaction_id = t.id
            WHERE (? IS NULL OR MONTH(t.datetime) = ?)
            AND (? IS NULL OR YEAR(t.datetime) = ?)
            AND (? IS NULL OR cat.id = ?)
            GROUP BY 1, 2, 3, 4
        ) grouped_totals
        "#,
    )
    .bind(month)
    .bind(month)
    .bind(year)
    .bind(year)
    .bind(category_id)
    .bind(category_id)
    .bind(month)
    .bind(month)
    .bind(year)
    .bind(year)
    .bind(category_id)
    .bind(category_id)
    .fetch_one(pool)
    .await?;

    let total_count = total_row.try_get::<i64, _>("total_count")? as u64;
    let offset = ((page - 1) * page_size) as i64;

    let totals = sqlx::query_as::<_, TransactionCategoryTotal>(
        r#"
        SELECT
            t.year,
            t.month,
            t.month_label,
            t.category_id,
            t.category,
            SUM(t.amount) AS total_amount
        FROM (
            SELECT
                CAST(YEAR(t.datetime) AS SIGNED) AS year,
                CAST(MONTH(t.datetime) AS SIGNED) AS month,
                CONCAT(YEAR(t.datetime), '-', LPAD(MONTH(t.datetime), 2, '0')) AS month_label,
                cat.id AS category_id,
                cat.name AS category,
                t.amount AS amount
            FROM transactions t
            INNER JOIN transactions_categories tc ON tc.transaction_id = t.id
            INNER JOIN categories cat ON tc.category_id = cat.id
            WHERE (? IS NULL OR MONTH(t.datetime) = ?)
            AND (? IS NULL OR YEAR(t.datetime) = ?)
            AND (? IS NULL OR cat.id = ?)

            UNION ALL

            SELECT
                CAST(YEAR(t.datetime) AS SIGNED) AS year,
                CAST(MONTH(t.datetime) AS SIGNED) AS month,
                CONCAT(YEAR(t.datetime), '-', LPAD(MONTH(t.datetime), 2, '0')) AS month_label,
                cat.id AS category_id,
                cat.name AS category,
                -st.amount AS amount
            FROM sub_transactions st
            INNER JOIN sub_transactions_categories stc ON stc.sub_transaction_id = st.id
            INNER JOIN categories cat ON stc.category_id = cat.id
            INNER JOIN transactions t ON st.transaction_id = t.id
            WHERE (? IS NULL OR MONTH(t.datetime) = ?)
            AND (? IS NULL OR YEAR(t.datetime) = ?)
            AND (? IS NULL OR cat.id = ?)
        ) t
        GROUP BY t.year, t.month, t.month_label, t.category_id, t.category
        ORDER BY t.year DESC, t.month DESC, total_amount ASC
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(month)
    .bind(month)
    .bind(year)
    .bind(year)
    .bind(category_id)
    .bind(category_id)
    .bind(month)
    .bind(month)
    .bind(year)
    .bind(year)
    .bind(category_id)
    .bind(category_id)
    .bind(page_size as i64)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok((totals, total_count))
}

pub async fn list_transaction_category_total_details(
    pool: &MySqlPool,
    month: u32,
    year: i32,
    category_id: i64,
) -> Result<Vec<TransactionCategoryTotalDetail>, anyhow::Error> {
    let details = sqlx::query_as::<_, TransactionCategoryTotalDetail>(
        r#"
        SELECT
            t.id,
            t.entry_type,
            t.year,
            t.month,
            t.month_label,
            t.description,
            t.datetime,
            t.note,
            t.category_id,
            t.category,
            t.amount
        FROM (
            SELECT
                trx.id AS id,
                'transaction' AS entry_type,
                CAST(YEAR(trx.datetime) AS SIGNED) AS year,
                CAST(MONTH(trx.datetime) AS SIGNED) AS month,
                CONCAT(YEAR(trx.datetime), '-', LPAD(MONTH(trx.datetime), 2, '0')) AS month_label,
                trx.description AS description,
                trx.datetime AS datetime,
                COALESCE(trx.note, '') AS note,
                cat.id AS category_id,
                cat.name AS category,
                trx.amount AS amount
            FROM transactions trx
            INNER JOIN transactions_categories tc ON tc.transaction_id = trx.id
            INNER JOIN categories cat ON tc.category_id = cat.id
            WHERE MONTH(trx.datetime) = ?
            AND YEAR(trx.datetime) = ?
            AND cat.id = ?

            UNION ALL

            SELECT
                st.id AS id,
                'sub-transaction' AS entry_type,
                CAST(YEAR(trx.datetime) AS SIGNED) AS year,
                CAST(MONTH(trx.datetime) AS SIGNED) AS month,
                CONCAT(YEAR(trx.datetime), '-', LPAD(MONTH(trx.datetime), 2, '0')) AS month_label,
                st.description AS description,
                trx.datetime AS datetime,
                '' AS note,
                cat.id AS category_id,
                cat.name AS category,
                -st.amount AS amount
            FROM sub_transactions st
            INNER JOIN sub_transactions_categories stc ON stc.sub_transaction_id = st.id
            INNER JOIN categories cat ON stc.category_id = cat.id
            INNER JOIN transactions trx ON st.transaction_id = trx.id
            WHERE MONTH(trx.datetime) = ?
            AND YEAR(trx.datetime) = ?
            AND cat.id = ?
        ) t
        ORDER BY t.amount ASC, t.id ASC
        "#,
    )
    .bind(month)
    .bind(year)
    .bind(category_id)
    .bind(month)
    .bind(year)
    .bind(category_id)
    .fetch_all(pool)
    .await?;

    Ok(details)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_total_model_shape() {
        let total = TransactionCategoryTotal {
            year: 2026,
            month: 4,
            month_label: "2026-04".to_string(),
            category_id: 2,
            category: "Bills".to_string(),
            total_amount: -350.12,
        };

        assert_eq!(total.year, 2026);
        assert_eq!(total.month, 4);
        assert_eq!(total.category, "Bills");
    }

    #[test]
    fn test_detail_model_shape() {
        let detail = TransactionCategoryTotalDetail {
            id: 1,
            entry_type: "sub-transaction".to_string(),
            year: 2026,
            month: 4,
            month_label: "2026-04".to_string(),
            description: "Water bill".to_string(),
            datetime: chrono::NaiveDateTime::parse_from_str(
                "2026-04-05 08:30:00",
                "%Y-%m-%d %H:%M:%S",
            )
            .unwrap(),
            note: "".to_string(),
            category_id: 2,
            category: "Bills".to_string(),
            amount: -45.0,
        };

        assert_eq!(detail.entry_type, "sub-transaction");
        assert_eq!(detail.category_id, 2);
        assert_eq!(detail.datetime.to_string(), "2026-04-05 08:30:00");
        assert_eq!(detail.amount, -45.0);
    }
}
