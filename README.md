# sqratch - modern sql gui and db management

Sqratch is a new SQL gui built for modern dev. Think Airtable for devs. If you need advanced DBA tools, TablePlus/DBeaver might be better for you. If you want to quickly grok what's in your database, quickly navigate between relations, view data with a type-aware UI, spin up internal dashboards with a click, and use modern AI to write scripts and debug issues, then you're in the right place.

## Features

- **Performance-First Design**: Native performance via Tauri with virtualized table rendering
- **Modern UI/UX**: Intuitive filtering, search, and relational data navigation
- **Smart Features**: AI-powered query generation and SQL autocompletion
- **Developer Experience**: Saved queries, snippets, and team collaboration

## CLI Usage

Sqratch can be launched directly from your project directory using the CLI, which automatically detects your database connection from your environment files based on your project configuration.

### Installation

```bash
# Install globally with npm
npm install -g sqratch

# Or with bun
bun install -g sqratch
```

### Usage

Navigate to your project directory and run:

```bash
# Using npx
npx sqratch

# Or using bunx
bunx sqratch
```

Sqratch will automatically:

1. Create a `.sqratch` directory in your project (if it doesn't exist)
2. Read your environment files (`.env`, `.env.local`, etc.)
3. Find database connection information based on your configuration
4. Launch the Sqratch application with your project path
5. Connect to your database

### Project Configuration

Sqratch projects use a `.sqratch` directory in the project root to store configuration and connection information. The main configuration file is `.sqratch/config.jsonc`, which follows this structure:

```jsonc
{
  // Environment variable name holding the connection string
  // Default: "DATABASE_URL"
  "connectionVariable": "DATABASE_URL",

  // Optional: Individual connection parameters as environment variables
  // Only used if connectionVariable doesn't exist or is empty
  "connectionParams": {
    "host": "DB_HOST",
    "port": "DB_PORT",
    "database": "DB_NAME",
    "user": "DB_USER",
    "password": "DB_PASSWORD",
  },

  // Project-specific settings
  "settings": {
    // Optional: Project name for display
    "projectName": "My Project",

    // Whether to save query history (default: true)
    "saveQueries": true,
  },
}
```

Sqratch will look for a `.env` file in the project root to load these environment variables. If not found, it will use the system environment variables.

### Project Structure

Sqratch creates the following structure in your project:

```
.sqratch/
  ├── config.json         # Project configuration
  ├── connections/        # Saved connection information
  └── queries/            # Saved SQL queries
```

This directory can be committed to your repository to share configurations and queries with your team.

## Development

```bash
# Install dependencies
bun install

# Run in development mode
bun dev

# Build for production
bun run build
```

## License

MIT
