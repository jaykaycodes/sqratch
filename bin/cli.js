#!/usr/bin/env node

const fs = require('node:fs')
const path = require('node:path')
const { spawn } = require('node:child_process')
const os = require('node:os')

// Simple .env file parser
function parseEnvFile(filePath) {
  try {
    const content = fs.readFileSync(filePath, 'utf8')
    const result = {}

    // Split by lines and process each line
    for (const line of content.split('\n')) {
      // Skip comments and empty lines
      if (!line || line.startsWith('#')) continue

      // Match key=value pattern, handling quotes
      const match = line.match(/^\s*([\w.-]+)\s*=\s*(.*)?\s*$/)
      if (!match) continue

      const key = match[1]
      let value = match[2] || ''

      // Remove quotes if present
      if (value.startsWith('"') && value.endsWith('"')) {
        value = value.slice(1, -1)
      } else if (value.startsWith("'") && value.endsWith("'")) {
        value = value.slice(1, -1)
      }

      // Handle escaped newlines
      value = value.replace(/\\n/g, '\n')

      result[key] = value
    }

    return result
  } catch (error) {
    return null
  }
}

// Find and load environment variables from various files
function loadEnvVars() {
  const cwd = process.cwd()
  const possibleFiles = [
    path.resolve(cwd, '.env'),
    path.resolve(cwd, '.env.local'),
    path.resolve(cwd, '.env.development'),
    path.resolve(cwd, '.env.development.local'),
  ]

  let envVars = {}

  for (const file of possibleFiles) {
    if (fs.existsSync(file)) {
      console.log(`Found env file: ${file}`)
      const vars = parseEnvFile(file)
      if (vars) {
        envVars = { ...envVars, ...vars }
      }
    }
  }

  return envVars
}

// Get or create the project-local Sqratch directory
function getProjectSqratchDir() {
  const cwd = process.cwd()
  const sqratchDir = path.join(cwd, '.sqratch')

  if (!fs.existsSync(sqratchDir)) {
    fs.mkdirSync(sqratchDir, { recursive: true })

    // Create subdirectories for organization
    fs.mkdirSync(path.join(sqratchDir, 'connections'), { recursive: true })
    fs.mkdirSync(path.join(sqratchDir, 'queries'), { recursive: true })
  }

  return sqratchDir
}

// Load or create project config file
function getProjectConfig() {
  const sqratchDir = getProjectSqratchDir()
  const configPath = path.join(sqratchDir, 'config.json')

  if (fs.existsSync(configPath)) {
    try {
      return JSON.parse(fs.readFileSync(configPath, 'utf8'))
    } catch (error) {
      console.warn('Error reading project config file, creating a new one')
    }
  }

  // Default project config
  const defaultConfig = {
    // The specific environment variable to use for the connection string
    connectionVariable: 'DATABASE_URL',
    // Optional: individual connection parameters if not using a connection string
    connectionParams: {
      host: 'DB_HOST',
      port: 'DB_PORT',
      database: 'DB_NAME',
      user: 'DB_USER',
      password: 'DB_PASSWORD',
    },
    // Project-specific settings
    settings: {
      projectName: path.basename(process.cwd()),
      saveQueries: true,
    },
  }

  // Save default config
  fs.writeFileSync(configPath, JSON.stringify(defaultConfig, null, 2))
  console.log(`Created default Sqratch config at ${configPath}`)
  console.log('You can customize this file to configure which environment variables to use')

  return defaultConfig
}

// Main function
async function main() {
  // Get project config
  const projectConfig = getProjectConfig()
  const projectSqratchDir = getProjectSqratchDir()

  // Load environment variables
  const envVars = loadEnvVars()

  // Try to find a database connection string using the configured variable
  let connectionString = null
  const configuredVar = projectConfig.connectionVariable

  if (envVars[configuredVar]) {
    connectionString = envVars[configuredVar]
    console.log(`Found connection string in ${configuredVar}`)
  } else {
    console.log(`No connection string found in ${configuredVar}`)

    // Try to construct from individual parameters if configured
    const params = projectConfig.connectionParams
    const host = envVars[params.host] || 'localhost'
    const port = envVars[params.port] || '5432'
    const database = envVars[params.database]
    const user = envVars[params.user]
    const password = envVars[params.password] || ''

    if (database && user) {
      connectionString = `postgresql://${user}:${password}@${host}:${port}/${database}`
      console.log('Constructed connection string from individual parameters')
    }
  }

  if (!connectionString) {
    console.log(
      'No database connection found using the configured variables in .sqratch/config.json',
    )
    console.log('Launching Sqratch without auto-connection...')
  } else {
    console.log('Storing connection information for Sqratch')

    // Store the connection info in the project directory
    const connectionInfo = {
      name: projectConfig.settings.projectName,
      connectionString,
      timestamp: Date.now(),
      path: process.cwd(),
    }

    // Save to the project connections directory
    fs.writeFileSync(
      path.join(projectSqratchDir, 'connections', 'current.json'),
      JSON.stringify(connectionInfo, null, 2),
    )
  }

  // Launch the Sqratch app
  console.log('Launching Sqratch...')

  // Determine the path to the Sqratch executable based on platform
  let sqratchPath

  if (process.platform === 'darwin') {
    // macOS
    sqratchPath = path.resolve(
      __dirname,
      '../',
      'src-tauri/target/release/bundle/macos/Sqratch.app/Contents/MacOS/Sqratch',
    )
  } else if (process.platform === 'win32') {
    // Windows
    sqratchPath = path.resolve(__dirname, '../', 'src-tauri/target/release/Sqratch.exe')
  } else {
    // Linux
    sqratchPath = path.resolve(__dirname, '../', 'src-tauri/target/release/sqratch')
  }

  // Pass the project path as an argument to the app
  const args = [`--project-path=${process.cwd()}`]

  if (fs.existsSync(sqratchPath)) {
    // Launch the native app
    const child = spawn(sqratchPath, args, {
      detached: true,
      stdio: 'ignore',
    })
    child.unref()
    console.log('Sqratch launched successfully!')
  } else {
    console.error('Sqratch executable not found at:', sqratchPath)
    console.error('Please make sure Sqratch is installed correctly.')
    process.exit(1)
  }
}

main().catch((err) => {
  console.error('Error launching Sqratch:', err)
  process.exit(1)
})
