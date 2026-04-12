use anyhow::{anyhow, bail};
use redis::aio::ConnectionManager;

use crate::models::categories::{Category, CategoryUpsert};
use sqlx::MySqlPool;

const CACHE_KEY: &str = "categories:all";

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

pub async fn list_categories(
    pool: &MySqlPool,
    cache: &mut ConnectionManager,
) -> Result<Vec<Category>, anyhow::Error> {
    if let Some(cached) = crate::cache::get(cache, CACHE_KEY).await {
        if let Ok(categories) = serde_json::from_str::<Vec<Category>>(&cached) {
            return Ok(categories);
        }
    }

    let categories = sqlx::query_as::<_, Category>(
        r#"
        SELECT id, name
        FROM categories
        ORDER BY name ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    if let Ok(json) = serde_json::to_string(&categories) {
        crate::cache::set(cache, CACHE_KEY, &json).await;
    }

    Ok(categories)
}

pub async fn create_category(
    pool: &MySqlPool,
    cache: &mut ConnectionManager,
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

    crate::cache::del(cache, CACHE_KEY).await;

    get_category_by_id(pool, result.last_insert_id() as i64).await
}

pub async fn update_category(
    pool: &MySqlPool,
    cache: &mut ConnectionManager,
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

    crate::cache::del(cache, CACHE_KEY).await;

    get_category_by_id(pool, category_id).await
}

pub async fn delete_category(
    pool: &MySqlPool,
    cache: &mut ConnectionManager,
    category_id: i64,
) -> Result<(), anyhow::Error> {
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

    crate::cache::del(cache, CACHE_KEY).await;

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
