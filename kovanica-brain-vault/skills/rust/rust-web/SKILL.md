---
name: rust-web
description: Use when building HTTP services in Rust: axum routing, extractors, middleware, state sharing, actix-web actors, warp filters, Tower middleware, Hyper basics, JSON serialization, and serving static content.
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [rust, web, HTTP, axum, actix-web, warp, tower, hyper, JSON, middleware]
    related_skills: [rust-async, rust-serialization, rust-error-handling]
---

# Rust Web

## Overview

Rust's web ecosystem is built on a few layers: **Hyper** (HTTP/1.1 and HTTP/2 client and server, low-level), **Tower** (a service-oriented middleware system), and framework crates that sit on top: **axum** (Ergonomic, Tower-based, currently the most popular), **actix-web** (mature, actor-based, high-performance), **warp** (filter-based, functional), **salvo** (less common but full-featured).

Axum is the recommended starting point for new projects in 2024-2025: it's built on Tower, has excellent ergonomics with extractors, integrates well with tokio, and has a clear migration path. Actix-web is a solid alternative with a large ecosystem.

## When to Use

- Building REST APIs, GraphQL servers, WebSocket services
- Serving HTTP endpoints with routing, extractors, middleware
- JSON request/response handling
- Serving static files, templates, or streaming responses
- Building gateway/proxy services with Tower middleware
- WebSocket servers (axum has built-in support)

**Don't use for:** gRPC (use `tonic` — built on axum/hyper but specialized), or FFI-based web servers (use the native binding of nginx or similar).

## Axum

### Minimal Server

```toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

```rust
use axum::{routing::get, Router, Json};
use serde::Serialize;

#[derive(Serialize)]
struct Hello {
    message: String,
}

async fn hello() -> Json<Hello> {
    Json(Hello { message: "Hello, world!".into() })
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(hello))
        .route("/hello/:name", get(hello_name));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn hello_name(axum::extract::Path(path): axum::extract::Path<String>) -> String {
    format!("Hello, {}!", path)
}
```

### Routing

```rust
use axum::{Router, routing::{get, post, put, delete, patch}, extract::Path};

let app = Router::new()
    // Static routes
    .route("/", get(root))
    .route("/about", get(about))

    // Path parameters
    .route("/users/:id", get(get_user))
    .route("/users/:id/posts/:post_id", get(get_post))

    // Multiple methods on same route
    .route("/items", get(list_items).post(create_item))

    // Nested routers (for grouping)
    .nest("/api", api_router())

    // Fallback for 404
    .fallback(four_oh_four);
```

Path parameters: `:id` captures a single segment. `*path` captures the rest (catch-all, returns `Path`).

### Extractors

Extractors pull data from the request into function arguments. Axum provides many built-in extractors:

```rust
use axum::{
    extract::{Path, Query, Json, State, Headers, CookieJar},
    http::StatusCode,
};

// Path parameter
async fn get_user(Path(id): Path<u64>) -> String {
    format!("User {}", id)
}

// Query parameters (parsed as a struct with Deserialize)
#[derive(serde::Deserialize)]
struct ListQuery {
    page: Option<u32>,
    limit: Option<u32>,
    sort: Option<String>,
}

async fn list_items(Query(q): Query<ListQuery>) {
    let page = q.page.unwrap_or(1);
    let limit = q.limit.unwrap_or(10);
    // ...
}

// JSON body
async fn create_item(Json(payload): Json<CreateItemRequest>) -> StatusCode {
    // payload: CreateItemRequest (Deserialize)
    StatusCode::CREATED
}

// Headers
async fn read_headers(Headers(headers): Headers) {
    if let Some(value) = headers.get("x-custom-header") {
        // ...
    }
}

// Cookies
async fn read_cookies(CookieJar jar: CookieJar) {
    if let Some(cookie) = jar.get("session") {
        // ...
    }
}

// Shared state
async fn handler(State(state): State<AppState>) {
    // state: AppState (cloned on each request, so it must be cheap to clone or Arc)
}
```

**Custom extractors:** implement `FromRequestParts` or `FromRequest` for your own types. Most cases are covered by built-in extractors and state.

### State Sharing

```rust
use std::sync::Arc;
use axum::extract::State;

#[derive(Clone)]
struct AppState {
    db: Arc<Database>,
    config: Config,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        db: Arc::new(Database::new()),
        config: Config::default(),
    };

    let app = Router::new()
        .route("/users", get(list_users))
        .with_state(state);
}

async fn list_users(State(state): State<AppState>) {
    // state.db and state.config available
}
```

`State` extracts are cloned on each request. If `AppState` is expensive to clone, wrap shared parts in `Arc` (as above) and keep cheap data in the struct itself.

### Response Types

```rust
use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};

// String / &str → 200 OK with text/plain
async fn text() -> &'static str {
    "Hello"
}

// JSON
async fn json() -> Json<Value> {
    Json(serde_json::json!({ "key": "value" }))
}

// Status code + body
async fn created() -> (StatusCode, String) {
    (StatusCode::CREATED, "Created".to_string())
}

// Custom response
async fn custom() -> Response {
    StatusCode::OK
        .with_header(axum::http::header::HeaderName::from_static("x-custom"), "value")
        .body(axum::body::Body::from("body"))
}

// Stream (for SSE or large responses)
use axum::body::{Body, StreamBody};
use tokio_stream::StreamExt;

async fn stream() -> StreamBody<impl futures_util::Stream<Item = Result<Bytes, std::io::Error>>> {
    // ...
}
```

### Error Handling

```rust
use axum::{
    http::StatusCode,
    response::IntoResponse,
    Json,
};

struct AppError {
    message: String,
    status: StatusCode,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.status, Json(serde_json::json!({
            "error": self.message
        }))).into_response()
    }
}

async fn handler() -> Result<Json<Value>, AppError> {
    if let Err(e) = do_something().await {
        return Err(AppError {
            message: e.to_string(),
            status: StatusCode::INTERNAL_SERVER_ERROR,
        });
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}
```

For application-level error types, implement `IntoResponse` and use `Result<_, AppError>` as the return type.

### Middleware

```rust
use axum::{
    middleware::{self, Next},
    extract::Request,
    response::Response,
};

// Layer-based middleware (applied to Router)
let app = Router::new()
    .route("/", get(handler))
    .layer(middleware::map_request(|request: Request| {
        // modify request before handler
        request
    }))
    .layer(middleware::map_response(|response: Response, request: Request| {
        // modify response after handler
        response
    }))
    .layer(middleware::from_fn_with_state(state, my_middleware_fn));

async fn my_middleware_fn(
    req: Request,
    next: Next,
) -> Response {
    // pre-processing
    let response = next.run(req).await;
    // post-processing
    response
}
```

**Tower layers:** axum uses Tower for middleware. Common layers:
- `tower_http::trace::TraceLayer` — request/response logging
- `tower_http::cors::CorsLayer` — CORS
- `tower_http::compression::CompressionLayer` — gzip/brotli
- `tower_http::auth::RequireAuthorizationLayer` — auth
- `tower_limit::rate::RateLimitLayer` — rate limiting (via tower-limit crate)

```rust
use tower_http::{
    trace::TraceLayer,
    cors::{CorsLayer, Any},
    compression::CompressionLayer,
};

let app = Router::new()
    .route("/", get(handler))
    .layer(TraceLayer::new_for_grplex())
    .layer(CompressionLayer::new())
    .layer(CorsLayer::new().allow_origin(Any));
```

### Serving Static Files

```rust
use tower_http::services::ServeDir;

let app = Router::new()
    .route("/api/*path", get(api_handler))
    .nest_service(
        "/static",
        ServeDir::new("assets")
            .append_index_html_on_directories(true)
    );
```

### WebSocket

```toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tower = "0.4"
```

```rust
use axum::{
    extract::WebSocketUpgrade,
    response::IntoResponse,
};
use tokio_tungstenite::tungstenite::Message;

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        while let Some(msg) = socket.recv().await {
            match msg {
                Message::Text(text) => {
                    // handle text
                }
                Message::Binary(data) => {
                    // handle binary
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    })
}
```

## Actix-Web

```toml
[dependencies]
actix-web = "4"
actix-rt = "2"
serde = { version = "1", features = ["derive"] }
```

```rust
use actix_web::{web, App, HttpServer, HttpResponse, Result};
use serde::Serialize;

#[derive(Serialize)]
struct Hello {
    message: String,
}

async fn hello() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(Hello { message: "Hello".into() }))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(hello))
            .route("/users/{id}", web::get().to(get_user))
    })
    .bind("0.0.0.0:3000")?
    .run()
    .await
}
```

Actix-web uses an actor runtime (`actix-rt`). Extractors work similarly but the API differs.

## Tower

Tower is a middleware/service framework: a `Service<Request>` that returns a `Future<Output = Response>`. It's the abstraction layer that axum, actix-web, and other services build on.

```rust
use tower::Service;
use tower_http::services::SpawnReadyLayer;

// A Tower service can be used directly:
async fn use_tower_service() {
    let mut service = Router::new()
        .route("/", get(handler))
        .into_make_service();

    // Serve with hyper directly
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, service).await.unwrap();
}
```

**When to use Tower directly:** when building middleware that's framework-agnostic, or when you need to compose services programmatically.

## Common Patterns

### REST CRUD

```rust
use axum::{
    extract::{Path, Json},
    http::StatusCode,
    routing::{get, post, put, delete},
    Router,
};

#[derive(Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

let app = Router::new()
    .route("/users", get(list_users).post(create_user))
    .route("/users/:id", get(get_user).put(update_user).delete(delete_user));
```

### Request Validation

```rust
use axum::Json;
use serde::Deserialize;
use validator::Validate;   // validator crate

#[derive(Deserialize, Validate)]
struct CreateUser {
    #[validate(length(min = 1, max = 100))]
    name: String,
    #[validate(email)]
    email: String,
}

async fn create_user(Json(payload): Json<CreateUser>) -> Result<StatusCode, (StatusCode, String)> {
    if let Err(e) = payload.validate() {
        return Err((StatusCode::BAD_REQUEST, e.to_string()));
    }
    // ...
    Ok(StatusCode::CREATED)
}
```

### Authentication

```rust
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

async fn auth_middleware(
    State(auth): State<Arc<AuthService>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = req.headers().get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    let user = match token {
        Some(t) => auth.validate_token(t).await?,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    // Attach user to request extensions
    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}
```

### CORS

```rust
use tower_http::cors::{CorsLayer, Any, Origin};

let cors = CorsLayer::new()
    .allow_origin(Any)   // or specific origins
    .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
    .allow_headers(Any);

let app = Router::new()
    .route("/", get(handler))
    .layer(cors);
```

## Verification Checklist

- [ ] Can set up an axum server with `Router`, routes, and `axum::serve`
- [ ] Can define routes with path parameters (`:id`) and catch-all (`*path`)
- [ ] Can use extractors: `Path`, `Query`, `Json`, `State`, `Headers`, `CookieJar`
- [ ] Can share state across handlers via `Arc` + `with_state`
- [ ] Can return JSON, text, status codes, and custom responses
- [ ] Can implement `IntoResponse` for custom error types
- [ ] Can add middleware via `.layer()` and write a simple middleware function
- [ ] Can use `tower_http` layers: `TraceLayer`, `CorsLayer`, `CompressionLayer`
- [ ] Can serve static files with `tower_http::services::ServeDir`
- [ ] Can set up a WebSocket endpoint with `WebSocketUpgrade`
- [ ] Can handle CORS properly (origin, methods, headers)
- [ ] Understands the difference between axum, actix-web, warp, and when to use each

## Common Pitfalls

1. **Blocking in async handlers.** Don't use `std::fs::read`, `std::net::TcpStream`, or other blocking I/O directly in an axum handler. Use `tokio::fs`, `tokio::net`, or `spawn_blocking` for blocking work.

2. **Not cloning `State` correctly.** `State` is cloned per request. If the state contains `Rc` or other non-`Send` types, the handlers can't be `Send` and fail to spawn on multi-threaded runtime. Use `Arc` for shared state.

3. **Returning `Result<_, Error>` without `IntoResponse` on `Error`.** Axum needs to know how to convert the error into a response. Implement `IntoResponse` or use `(StatusCode, String)` as the error type.

4. **CORS misconfiguration.** `allow_origin(Any)` is fine for development but dangerous in production. Configure specific origins in production.

5. **Forgetting to `await` futures in handlers.** Handlers are async — if you call an async function without `.await`, it doesn't run. This compiles but silently does nothing.

6. **Not setting content type.** Axum sets `application/json` for `Json<T>` responses. For custom responses, set headers explicitly.

7. **Using the wrong extractor for the data.** `Json` extracts a JSON body — use `Query` for query parameters, `Path` for path params, `Form` for form data.

8. **Shared mutable state without synchronization.** If handlers need to mutate shared state, use `Arc<Mutex<T>>` or `Arc<RwLock<T>>` (tokio's or std's). Be careful with lock scope across `.await`.

9. **Not handling errors in async middleware.** Middleware can fail — handle errors and return appropriate responses. Don't panic.

10. **Running in production without a reverse proxy.** Axum/actix-web servers are often exposed behind nginx/caddy for TLS, compression, and static file serving. Consider the deployment architecture.
