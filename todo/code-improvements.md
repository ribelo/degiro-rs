# Code Improvements Action Plan

## Phase 1: Eliminate Panics (Safety First)

- [ ] **Fix Lock Unwraps**
  - [ ] Replace all `state.read().unwrap()` with `unwrap_or_else(|e| e.into_inner())`
  - [ ] Replace all `state.write().unwrap()` with `unwrap_or_else(|e| e.into_inner())`
  - [ ] Search for remaining `.unwrap()` calls and assess each one
  - [ ] Use `expect()` only for "process is fucked anyway" cases (tokio runtime, etc)

- [ ] **Fix Result Unwraps**
  - [ ] `new_from_env()`(it should be named `load_from_env()`) - return Result instead of panicking on missing env vars
  - [ ] Check all `.unwrap()` in API methods
  - [ ] Add proper error handling for JSON parsing failures

## Phase 2: Add Retry Logic with Backon

- [ ] **Add backon dependency**
  ```toml
  backon = "search the web for the latest version"
  ```

- [ ] **Wrap HTTP execution**
  - [ ] Add retry logic to `execute_request()` in http.rs
  - [ ] Retry on: 500, 502, 503, 504, network errors
  - [ ] Don't retry on: 4xx errors (except 429)
  - [ ] Use exponential backoff starting at 100ms
  - [ ] Max 3 retries by default
  - [ ] Make retry policy configurable on client

- [ ] **Special handling for 429 (Rate Limit)**
  - [ ] Parse Retry-After header if present
  - [ ] Use longer backoff for rate limits

## Phase 3: Add Structured Logging

- [ ] **Add tracing dependency**
  ```toml
  tracing = "0.1"
  ```

- [ ] **Instrument key operations**
  - [ ] Add span for each API method
  - [ ] Log request details (method, path, NOT passwords)
  - [ ] Log response status and timing
  - [ ] Log retry attempts
  - [ ] Make log level configurable

- [ ] **Key events to log**
  - [ ] Authentication state changes
  - [ ] Session refresh
  - [ ] Rate limit hits
  - [ ] Retry attempts
  - [ ] Circuit breaker state (if we add it later)

## Phase 4: Fix Concurrent Operations

- [ ] **Portfolio fetching**
  - [ ] Replace `join().await` with `try_join_all`
  - [ ] Fail fast if any product fetch fails
  - [ ] Consider chunking large requests

- [ ] **Other concurrent operations**
  - [ ] Audit all uses of `join()`
  - [ ] Ensure proper error propagation
  - [ ] Add timeouts where appropriate

## Phase 5: Session Persistence

- [ ] **Save session state**
  - [ ] Create `~/.config/.degiro/` directory if not exists
  - [ ] Serialize SessionState to JSON
  - [ ] Encrypt with key derived from credentials
  - [ ] Save after successful login
  - [ ] Save after auth state changes

- [ ] **Load session on startup**
  - [ ] Check if session file exists
  - [ ] Decrypt and deserialize
  - [ ] Validate session is still valid (make test API call)
  - [ ] Clear if invalid

- [ ] **Security considerations**
  - [ ] Use proper encryption (not just base64)
  - [ ] Set file permissions to 0600
  - [ ] Clear on explicit logout
  - [ ] Add session expiry timestamp

## Phase 6: Health Check System

- [ ] **Add health check method**
  ```rust
  pub struct HealthStatus {
      session_valid: bool,
      auth_state: AuthState,
      last_successful_request: Option<Instant>,
      total_requests: u64,
      failed_requests: u64,
      rate_limit_remaining: u32,
      last_error: Option<(Instant, String)>,
  }
  ```

- [ ] **Track metrics**
  - [ ] Count successful/failed requests
  - [ ] Track last error with timestamp
  - [ ] Monitor rate limit usage
  - [ ] Add method to reset counters

## Quick Wins

- [ ] Add `#[must_use]` to builder methods
- [ ] Remove all commented-out code
- [ ] Add `thiserror` to all error enums that don't have it
- [ ] Add derive Debug where missing

## Testing Improvements

- [ ] **Add tests for retry logic**
  - [ ] Test successful retry after transient failure
  - [ ] Test max retry limit
  - [ ] Test non-retryable errors

- [ ] **Add tests for session persistence**
  - [ ] Test save/load cycle
  - [ ] Test invalid session handling
  - [ ] Test encryption

## Success Criteria

- [ ] Zero panics in production code paths
- [ ] All network operations have retry logic
- [ ] Failed operations have clear, actionable error messages
- [ ] Session survives process restart
- [ ] Can diagnose issues from logs alone
