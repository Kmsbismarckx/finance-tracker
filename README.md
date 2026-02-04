# Finance Tracker

Personal finance tracking API built with Rust, Axum, and PostgreSQL.

## Requirements

- Rust 1.75+
- Docker

## Quick Start

```bash
# Start PostgreSQL
docker compose up -d

# Run the server
cargo run --release

# Server runs on http://localhost:3000
```

## API Endpoints

### Accounts

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/accounts` | List all accounts |
| POST | `/api/accounts` | Create account |
| GET | `/api/accounts/:id` | Get account by ID |
| DELETE | `/api/accounts/:id` | Delete account |
| POST | `/api/accounts/:id/deposit` | Deposit money |
| POST | `/api/accounts/:id/withdraw` | Withdraw money |

### Examples

```bash
# Create account
curl -X POST http://localhost:3000/api/accounts \
  -H "Content-Type: application/json" \
  -d '{"name": "Wallet", "currency": "USD"}'

# Deposit
curl -X POST http://localhost:3000/api/accounts/<id>/deposit \
  -H "Content-Type: application/json" \
  -d '{"amount": 100.50}'

# Withdraw
curl -X POST http://localhost:3000/api/accounts/<id>/withdraw \
  -H "Content-Type: application/json" \
  -d '{"amount": 25.00}'

# List accounts
curl http://localhost:3000/api/accounts
```

## Development

```bash
# Run in dev mode
cargo run

# Run tests
cargo test

# Check code
cargo clippy
```

## Stop Services

```bash
# Stop the server
pkill -f finance-tracker

# Stop PostgreSQL
docker compose down

# Stop and remove data
docker compose down -v
```

## Project Structure

```
src/
├── main.rs              # Entry point
├── domain/              # Core business logic
│   ├── entities/        # Account, Transaction
│   └── errors.rs        # Domain errors
├── application/         # Use cases
│   ├── ports/           # Repository traits
│   ├── services/        # Business logic
│   └── dto/             # Request/Response DTOs
├── infrastructure/      # External implementations
│   ├── database/        # PostgreSQL repository
│   └── config.rs        # Configuration
└── presentation/        # API layer
    └── api/
        ├── handlers/    # HTTP handlers
        ├── routes.rs    # Routing
        └── error.rs     # Error handling
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | - | PostgreSQL connection string |
| `SERVER_HOST` | `127.0.0.1` | Server host |
| `SERVER_PORT` | `3000` | Server port |
| `RUST_LOG` | `info` | Log level |
