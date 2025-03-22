# Sqratch Test Project

This is a test project for developing and testing Sqratch application with a real project.

## Setup

This directory includes:

- A `.env` file with test database connection details
- Test project configuration in `.sqratch` (will be generated on first run)

## Usage

To test Sqratch with this project, run:

```bash
bun run dev:test
```

This will start Sqratch in development mode and automatically connect to this test project.

## Database

The test project is configured to connect to a PostgreSQL database with the following details:

- Host: localhost
- Port: 5432
- Database: test_db
- User: postgres
- Password: postgres

Make sure you have a PostgreSQL instance running with these credentials, or modify the `.env` file to match your test database.
