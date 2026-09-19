---
name: rust-databases
description: Use when working with databases in Rust: sqlx (async, compile-time checked SQL), diesel (ORM/query builder), rusqlite (SQLite), sea-orm, MongoDB, Redis, connection pools, migrations, and database testing patterns.
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [rust, databases, sqlx, diesel, rusqlite, sea-orm, mongodb, redis, migrations, connection-pools]
    related_skills: [rust-async, rust-web, rust-testing, rust-error-handling]
---

# Rust Databases

## Overview

Rust's database ecosystem offers choices between async-first (sqlx, sea-orm), synchronous ORM (diesel), embedded SQLite (rusqlite), document stores (mongodb), and caches (redis). The key decision: do you want compile-time SQL verification (sqlx macros), a Rust-native query builder (diesel), or raw SQL with runtime checking?

This skill covers the major options and common patterns: connection pooling, migrations, transactions, error handling, and testing with temporary databases.

## When to Use

- Choosing between sqlx, diesel, sea-orm, rusqlite for a new project
- Writing compile-time checked SQL with sqlx
- Building queries with diesel's query DSL
- Working with SQLite for embedded/local storage
- Connecting to PostgreSQL or MySQL in async contexts
- Using Redis for caching or pub/sub
- Setting up database migrations
- Writing tests that need a temporary database

**Don't use for:** ORMs that hide SQL entirely when you need query performance control, or when you specifically need an async runtime other than tokio (sqlx is tokio-only).

## sqlx — Async, Compile-Time Checked SQL

```toml
[dependencies]
sqlx = { version = "0.8", features = ["runtime-tokio", "tls-rustls", "postgres"] }
# or "mysql", "sqlite"
```

sqlx lets you write SQL queries as strings that are checked at compile time against a live database. Use `sqlx::query!` (returns a typed result) or `sqlx::query_as!` (maps to a struct).

### Setup and Migrations

```bash
cargo install sqlx-cli
sqlx init --database-url "postgres://user:pass@localhost/dbname"
sqlx migrate add create_users
```

Migrations go in `migrations/` directory. Run with `sqlx migrate run`.

### Query with Compile-Time Checking

```rust
use sqlx::PgPool;

// The macro connects to the database at compile time to verify the query.
// Requires DATABASE_URL env var at compile time.
let users = sqlx::query!(
    "SELECT id, name, email FROM users WHERE active = $1 ORDER BY created_at DESC",
    true
)
.fetch_all(&pool)
.fetch_all()
.await
.unwrap();

// users: Vec<{ id: i64, name: String, email: String }>
for user in users {
    println!("{} ({})", user.name, user.email);
}
```

**Requirement:** `DATABASE_URL` must be set at compile time for `query!`/`query_as!` macros. In CI, this means running a real database. Use `sqlx::query` (runtime-checked, no compile-time verification) if you can't provide a compile-time DB.

### Typed Mapping with `query_as!`

```rust
use sqlx::PgPool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct User {
    id: i64,
    name: String,
    email: String,
    active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

async fn get_active_users(pool: &PgPool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as!(
        User,
        "SELECT id, name, email, active, created_at FROM users WHERE active = $1",
        true
    )
    .fetch_all(pool)
    .await
}
```

`sqlx::FromRow` derives the mapping from column names to struct fields. `query_as!` maps the query result to the struct.

### Insert, Update, Delete

```rust
// Insert
let user = sqlx::query!(
    "INSERT INTO users (name, email, active, created_at) VALUES ($1, $2, $3, $4) RETURNING id",
    "Alice",
    "alice@example.com",
    true,
    chrono::Utc::now()
)
.fetch_one(&pool)
.await?;

let id = user.id;

// Update
sqlx::query!(
    "UPDATE users SET active = $1 WHERE id = $2",
    false,
    id
)
.execute(&pool)
.await?;

// Delete
sqlx::query!("DELETE FROM users WHERE id = $1", id)
.execute(&pool)
.await?;
```

### Transactions

```rust
async fn transfer_funds(
    pool: &PgPool,
    from_id: i64,
    to_id: i64,
    amount: i64,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query!(
        "UPDATE accounts SET balance = balance - $1 WHERE id = $2",
        amount,
        from_id
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "UPDATE accounts SET balance = balance + $1 WHERE id = $2",
        amount,
        to_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}
```

**Important:** Hold the transaction across `.await` points carefully. If the transaction is dropped before `commit`, it rolls back. Use `tokio::spawn` with care — a transaction doesn't survive across task boundaries unless explicitly managed.

### Connection Pooling

```rust
use sqlx::postgres::{PgPool, PgPoolOptions};

let pool = PgPoolOptions::new()
    .max_connections(10)
    .connect("postgres://user:pass@localhost/db")
    .await?;

// Use pool everywhere — it's Arc-backed, cheap to clone
async fn handler(pool: &PgPool) {
    sqlx::query!("SELECT 1").fetch_one(pool).await?;
}
```

Clone the pool freely — it's an `Arc` internally. Don't open/close connections per request; use the pool.

### Runtime-checked queries (when no compile-time DB)

```rust
// No compile-time checking, but works without DATABASE_URL at compile time
let rows = sqlx::query("SELECT id, name FROM users")
    .fetch_all(&pool)
    .await?;

for row in rows {
    let id: i64 = row.get("id");
    let name: String = row.get("name");
}
```

Use `query` (not `query!`) and extract columns manually. This is useful for CI without a database, or for dynamic queries.

### Error Handling

```rust
use sqlx::error::Error;

match result {
    Ok(v) => v,
    Err(Error::RowNotFound) => { /* no row matched */ }
    Err(Error::Database(e)) => { /* PostgreSQL/MySQL error codes */ }
    Err(e) => { /* other */ }
}
```

`sqlx::Error` has variants: `DbConnClosed`, `RowNotFound`, `Database`, `PoolClosed`, etc.

## diesel — Synchronous ORM / Query Builder

```toml
[dependencies]
diesel = { version = "2", features = ["postgres", "r2d2"] }
dotenvy = "0.15"
```

diesel is synchronous by default (async support is in diesel-async, a separate crate). It provides a Rust type system for your schema and a query DSL.

### Schema and Models

```rust
// schema.rs (generated by diesel print-schema or manually)
diesel::table! {
    users (id) {
        id -> BigInt,
        name -> Text,
        email -> Text,
        active -> Bool,
        created_at -> Timestamp,
    }
}

// models.rs
use diesel::prelude::*;
use diesel::Pg;

use crate::schema::users;

#[derive(Queryable, Selectable, Debug,Serialize)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub active: bool,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub name: String,
    pub email: String,
    pub active: bool,
}
```

### Queries

```rust
use diesel::prelude::*;
use diesel::Pg;

fn get_active_users(conn: &mut PgConnection) -> Result<Vec<User>, diesel::result::Error> {
    users::table
        .filter(users::active.eq(true))
        .order(users::created_at.desc())
        .load::<User>(conn)
}

fn find_user_by_id(conn: &mut PgConnection, id: i64) -> Result<User, diesel::result::Error> {
    users::table
        .filter(users::id.eq(id))
        .first::<User>(conn)
}

fn insert_user(conn: &mut PgConnection, new_user: NewUser) -> Result<User, diesel::result::Error> {
    diesel::insert_into(users::table)
        .values(&new_user)
        .get_result::<User>(conn)
}
```

### Transactions

```rust
use diesel::result::Error;

fn transfer_funds(conn: &mut PgConnection, from_id: i64, to_id: i64, amount: i64) -> Result<(), Error> {
    use diesel::connection::TransactionManager;

    conn.transaction::<_, Error, _>(|conn| {
        diesel::insert_into(balances::table)
            .values(&(from_id, -amount))
            .execute(conn)?;

        diesel::insert_into(balances::table)
            .values(&(to_id, amount))
            .execute(conn)?;

        Ok(())
    })
}
```

### Diesel 2.0+ Patterns

Diesel 2.0 introduced:
- `Selectable` trait for explicit column selection
- `PermittedFor` for type-safe joins
- Better derive macros

```rust
use diesel::prelude::*;

#[derive(Selectable, Queryable, Debug)]
#[diesel(table_name = users)]
struct UserView {
    id: i64,
    name: String,
}

let results = users::table
    .select(UserView::as_select())
    .filter(users::active.eq(true))
    .load::<UserView>(conn)?;
```

## rusqlite — SQLite

```toml
[dependencies]
rusqlite = { version = "0.32", features = ["bundled"] }   # bundled = compile SQLite in
```

rusqlite is the Rust binding to SQLite. `bundled` feature compiles SQLite from source (no system dependency).

### Basic Usage

```rust
use rusqlite::{Connection, Result, params};

let conn = Connection::open("my.db")?;

// Create table
conn.execute(
    "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT, email TEXT)",
    [],
)?;

// Insert
conn.execute(
    "INSERT INTO users (name, email) VALUES (?1, ?2)",
    params!["Alice", "alice@example.com"],
)?;

// Query
let mut stmt = conn.prepare("SELECT id, name, email FROM users WHERE active = ?1")?;
let rows = stmt.query_map(params![true], |row| {
    Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
})?;

for row in rows {
    let (id, name, email) = row?;
    println!("{id}: {name} <{email}>");
}
```

### Prepared Statements and Parameter Binding

Use `params!` macro for ergonomic parameter binding. Bind by index (`?1`, `?2`) or by name (`?name`) if using SQLite 3.8+.

### Transactions

```rust
let tx = conn.transaction()?;
tx.execute("UPDATE accounts SET balance = balance - ?1 WHERE id = ?2", params![100, 1])?;
tx.execute("UPDATE accounts SET balance = balance + ?1 WHERE id = ?2", params![100, 2])?;
tx.commit()?;
```

If the transaction is dropped, it rolls back.

### Bundled vs System SQLite

- `bundled` — compile SQLite from source, no system dependency, easier deployment.
- No `bundled` — link against system SQLite (libsqlite3). Requires the system library installed.

For most applications, `bundled` is preferred for portability.

## sea-orm — Async ORM

```toml
[dependencies]
sea-orm = { version = "1", features = ["runtime-tokio-native-tls", "with-postgres"] }
tokio = { version = "1", features = ["full"] }
```

sea-orm is an async ORM built on top of sqlx. It provides entity models, CRUD operations, and relations.

### Entity Definition

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, sea_orm::EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum Species {
    Cat,
    Dog,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "animals")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub species: Species,
    pub age: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

### CRUD

```rust
use sea_orm::{entity::prelude::*, query::Expr};

// Find by ID
let animal = Animal::find_by_id(1).one(&db).await?;

// Find with conditions
let animals = Animal::find()
    .filter(Animal::species.eq(Species::Cat))
    .filter(Animal::age.gt(5))
    .order_by_asc(Animal::name)
    .all(&db)
    .await?;

// Insert
let activemodel = AnimalActiveModel {
    name: Set("Whiskers".to_string()),
    species: Set(Species::Cat),
    age: Set(Some(3)),
    ..Default::default()
};
let animal = Animal::insert(activemodel).returning().one(&db).await?;

// Update
let animal = Animal::find_by_id(1).one(&db).await?;
if let Some(animal) = animal {
    let mutated = animal.into_active_model();
    mutated.age = Set(Some(4));
    let animal = Animal::update(mutated).returning().one(&db).await?;
}

// Delete
Animal::delete_by_id(1).exec(&db).await?;
```

### Relations

```rust
// Define relations in the enum
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "crate::animal::entity::Food")]
    Food,
}

// Load related data
let animal = Animal::find_by_id(1)
    .find_also_related(Food)
    .one(&db)
    .await?;
```

sea-orm handles JOINs via relations. `find_also_related` returns an optional related entity alongside the main entity.

## PostgreSQL with sqlx or tokio-postgres

For lower-level async PostgreSQL access without sqlx's compile-time checking:

```toml
[dependencies]
tokio-postgres = "0.7"
tokio = { version = "1", features = ["full"] }
```

```rust
use tokio_postgres::{NoTls, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres dbname=test",
        NoTls,
    )
    .await?;

    // Connection runs in background
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {e}");
        }
    });

    let rows = client
        .query("SELECT id, name FROM users WHERE active = $1", &[&true])
        .await?;

    for row in rows {
        let id: i64 = row.get(0);
        let name: &str = row.get(1);
        println!("{id}: {name}");
    }

    Ok(())
}
```

## MySQL with sqlx

```toml
[dependencies]
sqlx = { version = "0.8", features = ["runtime-tokio", "tls-rustls", "mysql"] }
```

Same API as PostgreSQL, different feature flag and connection string.

## Redis

```toml
[dependencies]
redis = "0.25"
tokio = { version = "1", features = ["full"] }
```

```rust
use redis::{AsyncCommands, RedisResult};

#[tokio::main]
async fn main() -> RedisResult<()> {
    let client = redis::Client::open("redis://127.0.0.1:6379")?;
    let mut con = client.get_async_connection().await?;

    // Set
    con.set("key", "value").await?;

    // Get
    let value: String = con.get("key").await?;
    println!("Got: {value}");

    // Hash
    con.hset("user:1", "name", "Alice").await?;
    con.hset("user:1", "email", "alice@example.com").await?;
    let name: String = con.hget("user:1", "name").await?;

    // List
    con.rpush("queue", "item1").await?;
    con.rpush("queue", "item2").await?;
    let item: String = con.lpop("queue").await?;

    // Pub/Sub (separate connection)
    let mut listener = client.get_async_connection().await?;
    // ...

    Ok(())
}
```

### Redis Connection Management

```rust
use redis::aio::ConnectionManager;   // auto-reconnects

let manager = redis::Client::open("redis://127.0.0.1:6379")?
    .get_connection_manager()
    .await?;
```

`ConnectionManager` handles reconnection automatically — useful for long-running services.

## MongoDB

```toml
[dependencies]
mongod = "2"
tokio = { version = "1", features = ["full"] }
```

```rust
use mongodb::{Client, Collection, bson::doc};
use mongodb::options::FindOptions;

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    let client = Client::with_uri_str("mongodb://localhost:27017").await?;
    let db = client.database("test");
    let collection: Collection<Document> = db.collection("users");

    // Insert one
    collection.insert_one(doc! { "name": "Alice", "age": 30 }, None).await?;

    // Find one
    let user = collection.find_one(doc! { "name": "Alice" }, None).await?;
    if let Some(user) = user {
        println!("Found: {:?}", user);
    }

    // Find many with options
    let mut find_options = FindOptions::default();
    find_options.sort = Some(doc! { "age": 1 });
    find_options.limit = Some(10);
    let cursor = collection.find(None, find_options).await?;
    while let Some(doc) = cursor.next().await {
        println!("User: {:?}", doc.unwrap());
    }

    Ok(())
}
```

## Connection String Patterns

| Database | Connection URL format |
|---|---|
| PostgreSQL | `postgres://user:pass@host:port/dbname?options` |
| MySQL | `mysql://user:pass@host:port/dbname` |
| SQLite (file) | `file:path/to/db.db` (sqlx) or just `path/to/db.db` (rusqlite) |
| SQLite (memory) | `memory:` (sqlx) or `Connection::open_in_memory()` (rusqlite) |
| Redis | `redis://[password@]host:port/db` or `rediss://` for TLS |
| MongoDB | `mongodb://[username:password@]host:port/[dbname]` |

## Testing with Temporary Databases

### PostgreSQL with testcontainers

```toml
[dev-dependencies]
testcontainers = { version = "0.15", features = ["postgres"] }
```

```rust
use testcontainers::clients::local::LocalDocker;
use testcontainers::images::postgres::Postgres;

async fn setup_test_db() -> (PgPool, LocalDocker) {
    let docker = LocalDocker::new();
    let container = docker.run(Postgres::default());
    let connection_string = container.connection_string("", "");
    // Use connection_string with sqlx::PgPool::connect
}
```

### SQLite in-memory for tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        Connection::open_in_memory().unwrap()
    }

    #[test]
    fn test_insert_and_query() {
        let conn = test_db();
        conn.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)", []).unwrap();
        conn.execute("INSERT INTO users (name) VALUES (?1)", params!["Test"]).unwrap();

        let mut stmt = conn.prepare("SELECT name FROM users").unwrap();
        let names: Vec<String> = stmt.query_map([], |row| row.get(0)).unwrap()
            .filter_map(|r| r.ok()).collect();

        assert_eq!(names, vec!["Test"]);
    }
}
```

### sqlx with offline mode

For CI without a database:

```bash
# Generate query metadata
cargo sqlx prepare

# In Cargo.toml, use `offline` feature
sqlx = { version = "0.8", features = ["runtime-tokio", "offline"] }
```

With offline mode, `query!`/`query_as!` macros use the generated `.sqlx` directory instead of a live database at compile time.

## Migrations

### sqlx-cli

```bash
sqlx migrate add create_users
# edits migrations/20240101000000_create_users.sql
sqlx migrate run       # run pending migrations
sqlx migrate revert   # revert last migration
sqlx migrate info     # show status
```

Migrations are applied in order; `sqlx migrate run` tracks applied versions in the database.

### diesel migration

```bash
diesel setup           # creates migrations/ and diesel.toml
diesel migration generate create_users
# edits migrations/*_create_users/{up,down}.sql
diesel migration run
diesel migration redo  # re-run last migration
```

### Manual migrations in code

Use a migrations table and apply SQL in order. Or use a crate like `refinery` or `rbaits`.

## Common Patterns

### Repository Pattern

```rust
use sqlx::PgPool;

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            "SELECT id, name, email FROM users WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn create(&self, new_user: NewUser) -> Result<User, sqlx::Error> {
        sqlx::query_as!(
            User,
            "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id, name, email",
            new_user.name,
            new_user.email
        )
        .fetch_one(&self.pool)
        .await
    }
}
```

### Connection per request (web apps)

```rust
use sqlx::PgPool;
use axum::extract::State;

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

// In handler:
async fn handler(State(state): State<AppState>) {
    sqlx::query!("SELECT 1").fetch_one(&state.db).await?;
}
```

Clone the pool into shared state. Don't open connections per request.

### Retry on connection failure

```rust
use sqlx::PgPool;
use std::time::Duration;
use tokio::time::sleep;

async fn connect_with_retry(url: &str) -> Result<PgPool, sqlx::Error> {
    let mut attempts = 0;
    loop {
        match PgPool::connect(url).await {
            Ok(pool) => return Ok(pool),
            Err(e) => {
                attempts += 1;
                if attempts > 5 {
                    return Err(e);
                }
                sleep(Duration::from_secs(1 << attempts)).await;
            }
        }
    }
}
```

## Verification Checklist

- [ ] Can set up sqlx with `query!` / `query_as!` macros and understand compile-time checking requirements
- [ ] Can write a Diesel schema, model, and query using the DSL
- [ ] Can open a SQLite database with rusqlite and execute queries with prepared statements
- [ ] Can perform CRUD with sea-orm entities
- [ ] Can use a connection pool (sqlx::PgPool) and understand that it's cheap to clone
- [ ] Can execute a transaction with rollback on drop
- [ ] Can connect to Redis asynchronously and perform basic operations
- [ ] Can connect to MongoDB and perform find/insert
- [ ] Can write compile-time-checked SQL and know when to fall back to runtime-checked
- [ ] Can set up database migrations with sqlx-cli or diesel
- [ ] Can write tests with an in-memory SQLite or a testcontainers PostgreSQL
- [ ] Understands the tradeoff between sqlx (compile-time SQL verification, async) and diesel (Rust DSL, synchronous)

## Common Pitfalls

1. **Missing DATABASE_URL at compile time for sqlx macros.** `query!` and `query_as!` need a live database at compile time. Use `DATABASE_URL` env var, or switch to offline mode or runtime-checked `query`.

2. **Using `query!`/`query_as!` when the database schema doesn't match.** The macros verify against the actual database. If your migration hasn't run, the compile-time check fails.

3. **Forgetting to commit transactions.** If the transaction handle is dropped before `commit`, it rolls back. Explicitly commit.

4. **Blocking the async runtime with synchronous database calls.** diesel is synchronous. If used in an async context, wrap in `spawn_blocking` or use diesel-async.

5. **Not using prepared statements for repeated queries.** rusqlite's `prepare` + `query_map` is more efficient than `execute` for repeated queries with different parameters.

6. **Forgetting that `sqlx::query!` returns a specific anonymous type.** The returned type is not `Vec<User>` — it's `Vec<{ id: i64, name: String }>`. Use `query_as!` to map to your struct, or rename fields with `AS` in SQL.

7. **Connection leaks.** Always return connections to the pool. Don't hold connections across `.await` unnecessarily (sqlx handles this, but be aware of transaction scope).

8. **SQL injection via string interpolation.** Never build SQL queries with string formatting of user input. Use parameterized queries (`$1`, `?1`, or named parameters).

9. **Not handling `RowNotFound` vs other errors in sqlx.** `query!` with `fetch_one` returns `RowNotFound` if no row matches. Distinguish from database errors.

10. **Assuming SQLite is production-ready for high concurrency.** SQLite has concurrency limitations (database-level locking). For high-write concurrency, use PostgreSQL/MySQL.
