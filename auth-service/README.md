# Auth Service - Setup Guide

## Prerequisites
- Docker and Docker Compose
- Rust and Cargo (for local development)
- PostgreSQL client (optional, for manual queries)

## Quick Start with Docker

### 1. Start the Database
```bash
cd /home/ssa/Documents/code/xendit
docker-compose up -d auth_db
```

### 2. Run Migrations
```bash
cd auth-service

# Install diesel CLI if not already installed
cargo install diesel_cli --no-default-features --features postgres

# Run migrations
diesel migration run
```

### 3. Run the Service
```bash
# Development mode
cargo run

# Or build and run release
cargo build --release
./target/release/xendit_auth
```

The service will be available at `http://localhost:8001`

## Testing the Service

### Register a User
```bash
curl -X POST http://localhost:8001/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "password123",
    "name": "Test User"
  }'
```

### Verify Email
```bash
curl -X POST http://localhost:8001/verify-email \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "code": "123456"
  }'
```

### Login
```bash
curl -X POST http://localhost:8001/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "password123"
  }'
```

## Environment Variables

Make sure to configure `.env`:
```
DATABASE_URL=postgresql://postgres:postgres@localhost:5433/auth_db
JWT_SECRET=your-secret-key
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password
SMTP_FROM=noreply@xendit.local
```

## Migration Commands

```bash
# Create a new migration
diesel migration generate migration_name

# Run pending migrations
diesel migration run

# Revert last migration
diesel migration revert

# Redo last migration
diesel migration redo
```
