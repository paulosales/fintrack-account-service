use crate::models::categories::Category;
use sqlx::MySqlPool;

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

#[cfg(test)]
mod tests {
    use crate::models::categories::Category;

    #[test]
    fn test_category_model() {
        let category = Category {
            id: 1,
            name: "Groceries".to_string(),
        };

        assert_eq!(category.id, 1);
        assert_eq!(category.name, "Groceries");
    }
}
