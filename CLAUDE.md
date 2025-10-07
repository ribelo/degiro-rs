# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Philosophy

Complexity is the enemy. Every abstraction must justify its existence.

Good code works and stays readable at 3am during an emergency. "Good taste" in engineering means knowing when to say no and recognizing elegant simplicity.

## Repository Overview

This is **degiro-ox**, a Rust client library for the DEGIRO trading platform API. It provides programmatic access to trading operations, portfolio management, and market data.

## Common Development Commands

```bash
# Build and test
cargo build
cargo test
cargo check
cargo clippy
cargo fmt

# Run specific tests
cargo test test_name
cargo test api::login

# Generate documentation
cargo doc --open

# MANDATORY before commit - Fix all clippy warnings
cargo clippy --fix --lib -p degiro-ox --tests --allow-dirty
```

## Architecture

The library uses an async/await design with Tokio, implements rate limiting (12 req/s), and handles multi-stage authentication with TOTP support.

### Key Modules
- `src/client.rs`: Main `Degiro` client struct with session management
- `src/session.rs`: Unified session state (all auth fields in one RwLock)
- `src/http.rs`: HTTP abstraction layer - `HttpRequest` builder and `HttpClient` trait
- `src/api/`: Endpoint implementations (login, orders, portfolio, quotes, etc.)
- `src/models/`: Data structures for API responses
- `src/paths.rs`: Centralized API endpoint URLs

### Recent Refactoring Patterns
- **HTTP calls**: Use `HttpRequest::get/post/put/delete(url).query().json()` with `self.request_json()`
- **No manual error handling**: The HTTP layer handles 401s, rate limiting, and error parsing
- **Session state**: Access via `self.session.field()` not separate atomics
- **Models organization**: Each type in its proper module (no util.rs dumping ground)
- **Retry logic**: All HTTP requests automatically retry transient failures (500, 502, 503, 504, 429)
- **Structured logging**: Use tracing for key operations (auth state changes, HTTP requests, retries)
- **Safe error handling**: No .unwrap() calls in production code paths

### Authentication Flow
1. Login with username/password
2. Generate TOTP token if 2FA is enabled
3. Maintain session using cookies
4. Handle automatic re-authentication on session expiry

## Development Notes

- The client can be initialized from environment variables: `DEGIRO_USERNAME`, `DEGIRO_PASSWORD`, `DEGIRO_TOTP_SECRET`
  - Use `Degiro::load_from_env()` which returns a Result instead of panicking
- All monetary values use `rust_decimal::Decimal` for precision
- Tests are co-located with implementation code in `#[cfg(test)]` modules
- All HTTP requests include automatic retry logic with exponential backoff
- Structured logging available via tracing crate for debugging and monitoring

## Important Constraints

- **This is a reverse-engineered API** - NEVER change how we communicate with DEGIRO's servers
- **Wire format is sacred** - Exchange enum must serialize as numeric strings ("663" not 663)
- **Use existing patterns** - When adding new endpoints, copy the pattern from similar methods
- **Avoid circular dependencies** - Use traits (like HttpClient) to extend functionality