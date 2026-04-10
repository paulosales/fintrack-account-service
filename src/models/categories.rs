use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct CategoryUpsert {
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_serialization() {
        let category = Category {
            id: 1,
            name: "Groceries".to_string(),
        };

        let json = serde_json::to_string(&category).unwrap();
        let deserialized: Category = serde_json::from_str(&json).unwrap();

        assert_eq!(category.id, deserialized.id);
        assert_eq!(category.name, deserialized.name);
    }

    #[test]
    fn test_category_upsert() {
        let payload = CategoryUpsert {
            name: "Bills".to_string(),
        };

        assert_eq!(payload.name, "Bills");
    }
}
