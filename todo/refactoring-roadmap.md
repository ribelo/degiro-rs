# Degiro-RS Refactoring Roadmap

## Phase 1: Documentation & Examples (Do First - Users Need This)

- [ ] **Create Examples Directory**
  - [ ] `examples/01_basic_auth.rs` - Simple login with environment variables
  - [ ] `examples/02_portfolio_overview.rs` - Display current positions and values
  - [ ] `examples/03_place_order.rs` - Safe order placement with validation
  - [ ] `examples/04_monitor_prices.rs` - Real-time price monitoring
  - [ ] `examples/05_export_transactions.rs` - Export transaction history
  - [ ] `examples/README.md` - Explain how to run examples, required env vars

- [ ] **API Documentation**
  - [ ] Document all public methods in `client.rs`
    - [ ] What each method does
    - [ ] Required auth level
    - [ ] Possible errors
    - [ ] Example usage
  - [ ] Document error types and when they occur
  - [ ] Create module-level docs explaining the architecture
  - [ ] Document the auth flow and state machine

- [ ] **README Overhaul**
  - [ ] Clear installation instructions
  - [ ] Quick start guide
  - [ ] Authentication setup (especially TOTP)
  - [ ] Common pitfalls and solutions
  - [ ] Link to examples

## Phase 2: Testing Infrastructure

- [ ] **Mock HTTP Layer**
  - [ ] Create `MockHttpClient` trait implementation
  - [ ] Record real API responses for test fixtures
  - [ ] Allow tests to run without credentials

- [ ] **Integration Test Suite**
  - [ ] Move inline tests to `tests/` directory
  - [ ] Create test helpers for common setup
  - [ ] Test error scenarios (401, 500, network errors)
  - [ ] Test auth state transitions
  - [ ] Test concurrent requests

- [ ] **CI/CD Setup**
  - [ ] GitHub Actions for tests
  - [ ] Clippy and fmt checks
  - [ ] Code coverage reporting
  - [ ] Example compilation checks

## Phase 3: Reliability Improvements

- [ ] **Retry Logic**
  - [ ] Implement exponential backoff for transient failures
  - [ ] Make retry configurable (max attempts, delays)
  - [ ] Different retry strategies for different error types
  - [ ] Don't retry on 4xx errors (except 429)

- [ ] **Better Async Error Handling**
  - [ ] Fix the `join().await` patterns that swallow errors
  - [ ] Use `try_join` for operations that should all succeed
  - [ ] Add timeout support for long-running operations

- [ ] **Connection Pool Management**
  - [ ] Configure connection reuse properly
  - [ ] Handle connection drops gracefully
  - [ ] Add connection health checks

## Phase 4: API Improvements

- [ ] **Builder Pattern Cleanup**
  - [ ] Remove the `#[builder(skip = ...)]` hacks
  - [ ] Make required fields explicit
  - [ ] Better default handling
  - [ ] Consider a simpler initialization pattern

- [ ] **URL Building**
  - [ ] Create proper URL builder that handles all the jsessionid nonsense
  - [ ] Centralize query parameter handling
  - [ ] Type-safe path construction

- [ ] **Request/Response Types**
  - [ ] Create dedicated types for each API endpoint
  - [ ] Move away from HashMaps where possible
  - [ ] Strong typing for all API interactions

## Phase 5: Feature Additions

- [ ] **Streaming Data Support**
  - [ ] WebSocket connection for real-time prices
  - [ ] Event stream for order updates
  - [ ] Proper reconnection logic

- [ ] **Caching Layer**
  - [ ] Cache product information (changes rarely)
  - [ ] Cache exchange information
  - [ ] Configurable cache TTLs
  - [ ] Cache invalidation on updates

- [ ] **Metrics & Monitoring**
  - [ ] Request/response timing
  - [ ] Rate limit usage tracking
  - [ ] Error rate monitoring
  - [ ] Optional OpenTelemetry support

## Phase 6: Security Enhancements

- [ ] **Credential Management**
  - [ ] Support for OS keyring storage
  - [ ] Encrypted credential file option
  - [ ] Remove credentials from memory after use
  - [ ] Session token refresh before expiry

- [ ] **Audit Logging**
  - [ ] Log all trading operations
  - [ ] Configurable log levels
  - [ ] Sensitive data redaction
  - [ ] Structured logging support

## Phase 7: Performance Optimization

- [ ] **Batch Operations**
  - [ ] Batch product info requests
  - [ ] Batch quote requests
  - [ ] Parallel request optimization

- [ ] **Memory Usage**
  - [ ] Reduce cloning where possible
  - [ ] Use Cow for rarely-modified strings
  - [ ] Stream large responses instead of loading to memory

## Quick Wins (Can Do Anytime)

- [ ] Add `#[must_use]` to builder methods
- [ ] Add `Debug` implementations where missing
- [ ] Remove remaining `unwrap()` calls
- [ ] Add `Display` implementations for public types
- [ ] Consistent naming (pick `account_id` or `client_id`, not both)
- [ ] Remove commented-out code
- [ ] Add repository metadata to Cargo.toml

## Future Considerations

- [ ] **API Version Support**
  - [ ] Track Degiro API changes
  - [ ] Support multiple API versions
  - [ ] Deprecation warnings

- [ ] **Alternative Backends**
  - [ ] Support for other brokers with similar APIs
  - [ ] Abstract the broker-specific parts
  - [ ] Plugin architecture

## Success Metrics

- Zero panics in production code
- All public APIs documented
- 80%+ test coverage
- Examples for all major use cases
- < 5 minute onboarding for new users
- Proper error messages that tell you how to fix the problem