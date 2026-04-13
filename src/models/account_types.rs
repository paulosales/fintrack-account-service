use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AccountType {
    pub id: i64,
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountTypeUpsert {
    pub code: String,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_type_serialization() {
        let account_type = AccountType {
            id: 1,
            code: "CHECKING".to_string(),
            name: "Checking".to_string(),
        };

        let json = serde_json::to_string(&account_type).unwrap();
        let deserialized: AccountType = serde_json::from_str(&json).unwrap();

        assert_eq!(account_type.id, deserialized.id);
        assert_eq!(account_type.code, deserialized.code);
        assert_eq!(account_type.name, deserialized.name);
    }
}
