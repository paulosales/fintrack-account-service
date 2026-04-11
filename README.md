# Account Service

A Rust-based REST API service for managing personal finance data using the [Axum](https://github.com/tokio-rs/axum) web framework and a MySQL database.

## Features

- RESTful API for accounts, transactions, sub-transactions, categories, transaction types, budgets, budget setups, and transaction-category totals
- MySQL integration via SQLx
- Paginated JSON responses with structured error handling
- CORS enabled for local development
- Environment-based configuration

## API Endpoints

The server runs on `http://0.0.0.0:3001`.

### Accounts
| Method | Path | Description |
|--------|------|-------------|
| GET | `/accounts` | List all accounts |

### Transactions
| Method | Path | Description |
|--------|------|-------------|
| GET | `/transactions` | List transactions (supports pagination & filters) |
| POST | `/transactions` | Create a transaction |
| PUT | `/transactions/{id}` | Update a transaction |
| DELETE | `/transactions/{id}` | Delete a transaction |

### Sub-Transactions
| Method | Path | Description |
|--------|------|-------------|
| GET | `/transactions/{id}/sub_transactions` | List sub-transactions for a transaction |
| POST | `/transactions/{id}/sub_transactions` | Create a sub-transaction |
| PUT | `/transactions/{transaction_id}/sub_transactions/{id}` | Update a sub-transaction |
| DELETE | `/transactions/{transaction_id}/sub_transactions/{id}` | Delete a sub-transaction |

### Categories
| Method | Path | Description |
|--------|------|-------------|
| GET | `/categories` | List all categories |
| POST | `/categories` | Create a category |
| PUT | `/categories/{id}` | Update a category |
| DELETE | `/categories/{id}` | Delete a category |

### Transaction Types
| Method | Path | Description |
|--------|------|-------------|
| GET | `/transaction-types` | List all transaction types |

### Budgets
| Method | Path | Description |
|--------|------|-------------|
| GET | `/budgets` | List budget month totals |
| POST | `/budgets` | Create a budget |
| GET | `/budgets/details` | List budget details |
| POST | `/budgets/generate` | Generate budgets |
| PUT | `/budgets/{id}` | Update a budget |
| DELETE | `/budgets/{id}` | Delete a budget |

### Budget Setups
| Method | Path | Description |
|--------|------|-------------|
| GET | `/budget-setups` | List budget setups |
| POST | `/budget-setups` | Create a budget setup |
| PUT | `/budget-setups/{id}` | Update a budget setup |
| DELETE | `/budget-setups/{id}` | Delete a budget setup |

### Transaction Category Totals
| Method | Path | Description |
|--------|------|-------------|
| GET | `/transaction-category-totals` | List category totals |
| GET | `/transaction-category-totals/details` | List category total details |

## Development

### Prerequisites

- Rust 1.70+
- MySQL database

### Environment Setup

Create a `.env` file in the project root:

```env
DATABASE_URL=mysql://username:password@localhost:3306/fintrack
```

### Running the Service

```bash
cargo run
```

The server will start on `http://0.0.0.0:3001`.

## Formatting

This project uses `rustfmt` as its code formatter.

```bash
# Install
rustup component add rustfmt

# Format
cargo fmt

# Check (CI / pre-commit)
cargo fmt --check
```

VS Code is configured to use `rust-analyzer` as the default formatter for Rust files and to format on save.

## Testing

```bash
# Run all tests
cargo test

# Run only unit tests by module
cargo test models::
cargo test services::
cargo test controllers::
cargo test db::

# Run integration tests
cargo test --test integration_tests
```

### Test Coverage
- **Models**: Struct serialization/deserialization and validation
- **Services**: Business logic and data manipulation
- **Controllers**: HTTP request handling and response formatting
- **Database**: Connection configuration and environment variable handling
- **Integration**: Cross-module functionality and JSON response structures

## Project Structure

```
src/
├── main.rs
├── controllers/
│   ├── account_controller.rs
│   ├── budget_controller.rs
│   ├── budget_setup_controller.rs
│   ├── category_controller.rs
│   ├── transaction_category_total_controller.rs
│   ├── transaction_controller.rs
│   └── transaction_type_controller.rs
├── db/
│   └── connection.rs
├── models/
│   ├── accounts.rs
│   ├── budget_setups.rs
│   ├── budgets.rs
│   ├── categories.rs
│   ├── pagination.rs
│   ├── sub_transactions.rs
│   ├── transaction_category_totals.rs
│   ├── transaction_types.rs
│   └── transactions.rs
├── routes/
│   ├── account_routes.rs
│   ├── budget_routes.rs
│   ├── budget_setup_routes.rs
│   ├── category_routes.rs
│   ├── transaction_category_total_routes.rs
│   ├── transaction_routes.rs
│   └── transaction_type_routes.rs
└── services/
    ├── account_service.rs
    ├── budget_service.rs
    ├── budget_setup_service.rs
    ├── category_service.rs
    ├── transaction_category_total_service.rs
    ├── transaction_service.rs
    └── transaction_type_service.rs

tests/
└── integration_tests.rs
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `axum` | Web framework |
| `tokio` | Async runtime |
| `sqlx` | MySQL access with compile-time query checking |
| `serde` / `serde_json` | Serialization / deserialization |
| `chrono` | Date and time handling |
| `dotenv` | Environment variable loading |
| `tower-http` | CORS middleware |
| `uuid` | UUID generation |
| `anyhow` | Error handling |