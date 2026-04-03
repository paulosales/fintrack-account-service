use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Account {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub account_type_id: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_account_serialization() {
        let account = Account {
            id: 1,
            code: "CHK-001".to_string(),
            name: "Checking Account".to_string(),
            account_type_id: 1,
        };

        let json = serde_json::to_string(&account).unwrap();
        let deserialized: Account = serde_json::from_str(&json).unwrap();

        assert_eq!(account.id, deserialized.id);
        assert_eq!(account.code, deserialized.code);
        assert_eq!(account.name, deserialized.name);
        assert_eq!(account.account_type_id, deserialized.account_type_id);
    }
}
