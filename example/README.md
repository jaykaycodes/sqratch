All you need for a local sqratch project is a `.sqratch/config.json` file to tell sqratch where to find your database:

```ts
{
  "name": "my-project" // Optional, will be inferred from parent directory or db name
	"db": "postgres://postgres:postgres@localhost:5432/postgres" // Required, connection string for the database
}
```

You can also pass a relative path to an env file:

```ts
{
  "db": "../.env" // Will load DATABASE_URL from .env
}
```

Or specify the name of the environment variable:

```ts
{
  "db": "../.env|DB_URL" // Will load DATABASE_URL from .env
}
```
