use anyhow::{anyhow, bail};

use crate::models::categories::{Category, CategoryUpsert};
use sqlx::MySqlPool;

async fn get_category_by_id(pool: &MySqlPool, category_id: i64) -> Result<Category, anyhow::Error> {
    let category = sqlx::query_as::<_, Category>(
        r#"
        SELECT id, name
        FROM categories
        WHERE id = ?
        "#,
    )
    .bind(category_id)
    .fetch_optional(pool)
    .await?;

    category.ok_or_else(|| anyhow!("Category {} not found", category_id))
}

pub async fn list_categories(pool: &MySqlPool) -> Result<Vec<Category>, anyhow::Error> {
    let categories = sqlx::query_as::<_, Category>(
        r#"
        SELECT id, name
        FROM categories
        ORDER BY name ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(categories)
}

pub async fn create_category(
    pool: &MySqlPool,
    payload: CategoryUpsert,
) -> Result<Category, anyhow::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO categories (name)
        VALUES (?)
        "#,
    )
    .bind(payload.name)
    .execute(pool)
    .await?;

    get_category_by_id(pool, result.last_insert_id() as i64).await
}

pub async fn update_category(
    pool: &MySqlPool,
    category_id: i64,
    payload: CategoryUpsert,
) -> Result<Category, anyhow::Error> {
    let result = sqlx::query(
        r#"
        UPDATE categories
        SET name = ?
        WHERE id = ?
        "#,
    )
    .bind(payload.name)
    .bind(category_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        bail!("Category {} not found", category_id);
    }

    get_category_by_id(pool, category_id).await
}

pub async fn delete_category(pool: &MySqlPool, category_id: i64) -> Result<(), anyhow::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM categories
        WHERE id = ?
        "#,
    )
    .bind(category_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        bail!("Category {} not found", category_id);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::models::categories::{Category, CategoryUpsert};

    #[test]
    fn test_category_model() {
        let category = Category {
            id: 1,
            name: "Groceries".to_string(),
        };

        assert_eq!(category.id, 1);
        assert_eq!(category.name, "Groceries");
    }

    #[test]
    fn test_category_upsert_model() {
        let payload = CategoryUpsert {
            name: "Housing".to_string(),
        };

        assert_eq!(payload.name, "Housing");
    }
}
