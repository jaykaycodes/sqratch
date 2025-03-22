# Sqratch Backend Development Guide

This document provides an overview of the Sqratch backend architecture, which is built using Tauri and Rust. It's intended to help developers understand how the codebase is organized and how the different components interact.

## Architecture Overview

The Sqratch backend is structured using a modular approach with clear separation of concerns:

```
src-tauri/
├── src/
│   ├── main.rs             # Application entry point
│   ├── lib.rs              # Main library code and app initialization
│   ├── commands.rs         # Tauri commands exposed to the frontend
│   ├── startup.rs          # Application startup logic
│   ├── cli.rs              # CLI argument handling
│   ├── db/                 # Database functionality
│   │   ├── mod.rs          # Module definition
│   │   ├── manager.rs      # Database connection manager
│   │   ├── types.rs        # Data types and models
│   │   ├── errors.rs       # Error handling
│   │   ├── utils/          # Utility functions
│   │   └── clients/        # Database client implementations
│   │       ├── mod.rs      # Module definition
│   │       ├── common.rs   # Common database client interface
│   │       ├── postgres.rs # PostgreSQL client implementation
│   │       ├── mysql.rs    # MySQL client implementation
│   │       └── sqlite.rs   # SQLite client implementation
│   └── bin/                # Binary executables
├── Cargo.toml              # Rust dependencies
└── tauri.conf.json         # Tauri configuration
```

## Core Components

### Application Entry Point

- `main.rs` - The entry point for the Tauri application. It simply calls the `run()` function from `lib.rs`.

### Application Setup

- `lib.rs` - Sets up the Tauri application, including plugins, state management, and command handlers.
  - Creates the central `DatabaseManager` instance
  - Configures Tauri plugins (fs, cli, opener)
  - Sets up application state with database manager
  - Processes CLI arguments at startup
  - Registers command handlers for frontend invocation

### Command Handler

- `commands.rs` - Implements Tauri commands that can be invoked from the frontend.
  - Provides a bridge between the frontend and the Rust backend
  - Each function is decorated with `#[tauri::command]` and handles a specific task
  - Commands include database operations like connecting, querying, and managing connections

### Startup Logic

- `startup.rs` - Contains logic for application startup procedures.
  - Configures file system permissions
  - Handles project path loading
  - Attempts to discover and connect to databases in project directories

### CLI Handling

- `cli.rs` - Manages command-line interface arguments.
  - Processes CLI arguments like project paths
  - Extracts positional arguments for opening projects
  - Ensures the app window is visible after processing commands

## Database Module

The database module (`db/`) is the core of the backend functionality, providing a flexible interface for connecting to and interacting with different database systems.

### Database Manager

- `manager.rs` - Manages database connections and client instances.
  - Stores connection configurations
  - Creates and manages database client instances
  - Handles connection lifecycle (connect, disconnect, etc.)
  - Provides methods for query execution
  - Includes utilities for loading connection configs from projects

### Types

- `types.rs` - Defines data structures used throughout the database module.
  - `DatabaseType` - Enumeration of supported database types
  - `ConnectionInfo` - Connection configuration data
  - `QueryResult` - Structure for query results
  - `ColumnDefinition` - Column metadata
  - `Row` - Result row structure
  - Various schema-related types for tables, views, and functions

### Error Handling

- `errors.rs` - Custom error types for database operations.
  - `DbError` - Enumeration of database error types
  - `DbResult` - Result type alias for database operations

### Database Clients

- `clients/common.rs` - Defines traits for database clients and transactions.

  - `DatabaseClient` trait - Common interface for all database clients
  - `Transaction` trait - Interface for database transactions

- Client Implementations:
  - `postgres.rs` - PostgreSQL client implementation
  - `mysql.rs` - MySQL client implementation
  - `sqlite.rs` - SQLite client implementation

Each client implements the `DatabaseClient` trait, providing a consistent interface while handling database-specific details.

## Application State

The application maintains state through the `AppState` struct defined in `lib.rs`:

```rust
pub struct AppState {
    db_manager: Arc<DatabaseManager>,
}
```

This state is managed by Tauri and injected into command handlers, providing access to the database manager throughout the application.

## Command Flow

1. The frontend invokes a Tauri command (e.g., `execute_query`)
2. The command handler in `commands.rs` receives the request
3. The handler accesses the application state to get the database manager
4. The database manager delegates to the appropriate database client
5. Results are returned to the frontend as serialized JSON

## CLI Functionality

The app supports command-line usage patterns:

- Opening the app without arguments: `sqratch`
- Opening a specific project: `sqratch /path/to/project`

The CLI functionality is implemented through:

1. Argument processing in `cli.rs`
2. Connection detection in `startup.rs`
3. Database loading through the database manager

## Project Integration

The application can detect and load database connections from a project directory:

1. When a project path is provided via CLI, the app looks for a `.sqratch` directory
2. Connection configurations are loaded from this directory
3. The app automatically connects to discovered databases

## Error Handling

Error handling is implemented through:

- Custom error types in `db/errors.rs`
- Result types that propagate errors up the call chain
- Error conversion to strings for frontend communication

## Development Workflow

When extending the application:

1. For new database types:

   - Add a new enum variant to `DatabaseType`
   - Create a new client implementation in `db/clients/`
   - Update the client factory in `manager.rs`

2. For new frontend commands:

   - Add a new function in `commands.rs` with the `#[tauri::command]` attribute
   - Register the command in the `invoke_handler` in `lib.rs`

3. For CLI enhancements:
   - Modify argument handling in `cli.rs`
   - Update startup logic in `startup.rs` if needed

## Configuration

- `Cargo.toml` - Rust dependencies and build configuration
- `tauri.conf.json` - Tauri application configuration
  - Defines window properties
  - Configures allowed APIs
  - Sets up file system permissions

## Security Considerations

- File system permissions are carefully configured in `startup.rs`
- Database credentials are stored in memory only
- The application follows the principle of least privilege
