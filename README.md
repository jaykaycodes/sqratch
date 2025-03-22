# Sqratch - Modern SQL Client

Sqratch is a high-performance SQL client and database management tool built with Tauri, React, and TypeScript. It aims to modernize database workflows with AI-powered features, intuitive filtering, and seamless collaboration.

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

Sqratch uses a project-local configuration file at `.sqratch/config.json` that can be committed to your repository and shared with your team. This ensures everyone on the team uses the same connection variables.

The first time you run Sqratch in a project, it will create a default configuration:

```json
{
  "connectionVariable": "DATABASE_URL",
  "connectionParams": {
    "host": "DB_HOST",
    "port": "DB_PORT",
    "database": "DB_NAME",
    "user": "DB_USER",
    "password": "DB_PASSWORD"
  },
  "settings": {
    "projectName": "your-project-name",
    "saveQueries": true
  }
}
```

You can customize this file to:

- Specify which environment variable contains your connection string
- Define which variables to use for individual connection parameters
- Set project-specific settings

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
