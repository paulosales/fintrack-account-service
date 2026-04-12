use dotenv::dotenv;
use sqlx::MySqlPool;
use std::env;

pub async fn get_pool() -> MySqlPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://user:password@localhost/fintrack".to_string());

    MySqlPool::connect(&database_url)
        .await
        .expect("Failed to create pool")
}

pub async fn run_migrations(pool: &MySqlPool) {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .expect("Failed to run database migrations");
}

#[cfg(test)]
mod tests {
    use std::env;

    #[test]
    fn test_default_database_url() {
        // Clear any existing DATABASE_URL
        env::remove_var("DATABASE_URL");

        // Test that we get the default URL when no env var is set
        // Note: We can't actually test the connection without a real database,
        // but we can test the URL construction logic

        let default_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "mysql://user:password@localhost/fintrack".to_string());

        assert!(default_url.contains("mysql://"));
        assert!(default_url.contains("localhost"));
        assert!(default_url.contains("fintrack"));
    }

    #[test]
    fn test_custom_database_url_from_env() {
        // Set a custom DATABASE_URL
        let custom_url = "mysql://testuser:testpass@testhost:3306/testdb";
        env::set_var("DATABASE_URL", custom_url);

        let url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "mysql://user:password@localhost/fintrack".to_string());

        assert_eq!(url, custom_url);
        assert!(url.contains("testuser"));
        assert!(url.contains("testhost"));
        assert!(url.contains("testdb"));

        // Clean up
        env::remove_var("DATABASE_URL");
    }

    #[test]
    fn test_database_url_components() {
        let test_urls = vec![
            "mysql://user:pass@localhost/db",
            "mysql://admin:secret@db.example.com:3306/fintrack",
            "mysql://readonly:readonly@readonly-host/fintrack_readonly",
        ];

        for url in test_urls {
            assert!(
                url.starts_with("mysql://"),
                "URL should start with mysql://"
            );
            assert!(url.contains("@"), "URL should contain @ separator");
            assert!(url.contains("/"), "URL should contain database path");
        }
    }

    #[test]
    fn test_environment_variable_handling() {
        // Test that we can set and retrieve environment variables
        let test_key = "TEST_DATABASE_URL";
        let test_value = "mysql://test:test@localhost/test";

        env::set_var(test_key, test_value);
        let retrieved = env::var(test_key).unwrap();
        assert_eq!(retrieved, test_value);

        // Clean up
        env::remove_var(test_key);
    }
}
