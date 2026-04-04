# Account Service

A Rust-based REST API service for managing financial account transactions using Axum web framework and MySQL database.

## Features

- RESTful API for transaction management
- MySQL database integration with SQLx
- JSON API responses with proper error handling
- Environment-based configuration

## API Endpoints

### GET /transactions
Retrieve transactions with optional account filtering.

**Query Parameters:**
- `account_id` (optional): Filter transactions by account ID

**Example:**
```bash
curl "http://localhost:3000/transactions"
curl "http://localhost:3000/transactions?account_id=123"
```

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": 1,
      "account_id": 123,
      "transaction_type_id": 2,
      "datetime": "2024-01-15T10:30:00",
      "amount": 150.50,
      "description": "Grocery shopping",
      "note": "Weekly groceries",
      "fingerprint": "abc123def456"
    }
  ],
  "count": 1
}
```

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

The server will start on `http://localhost:3000`.

## Formatting

This backend uses `rustfmt` as its code formatter.

### Install the Formatter

```bash
rustup component add rustfmt
```

### Format the Code

```bash
cargo fmt
```

### Check Formatting in CI or Before Commit

```bash
cargo fmt --check
```

VS Code is configured to use `rust-analyzer` as the default formatter for Rust files and to format on save.

## Testing

### Prerequisites
The project includes comprehensive unit tests that require the test dependencies.

### Running Unit Tests
Run all tests with:
```bash
cargo test
```

### Test Coverage
The test suite covers:
- **Models**: Transaction struct serialization/deserialization and validation
- **Services**: Transaction business logic and data manipulation
- **Controllers**: HTTP request handling and response formatting
- **Database**: Connection configuration and environment variable handling
- **Integration**: Cross-module functionality and JSON response structures

### Running Specific Tests
```bash
# Test only the models
cargo test models::

# Test only the services
cargo test services::

# Test only the controllers
cargo test controllers::

# Test only the database module
cargo test db::

# Run integration tests
cargo test --test integration_tests
```

## Project Structure

```
src/
├── main.rs              # Application entry point
├── models/
│   └── transactions.rs  # Transaction data models
├── services/
│   └── transaction_service.rs  # Business logic layer
├── controllers/
│   └── transaction_controller.rs  # HTTP request handlers
├── routes/
│   └── transaction_routes.rs  # Route definitions
└── db/
    └── connection.rs    # Database connection management

tests/
└── integration_tests.rs # Integration tests
```

## Dependencies

- **axum**: Web framework for building the API
- **tokio**: Async runtime
- **sqlx**: Database access with compile-time query checking
- **serde**: Serialization/deserialization
- **chrono**: Date and time handling
- **dotenv**: Environment variable management