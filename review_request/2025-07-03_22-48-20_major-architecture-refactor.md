# Code Review Request: Major Architecture Refactor

## Summary of Changes

### What was done:
- Introduced unified HTTP abstraction layer with automatic retry logic and rate limiting
- Implemented comprehensive session state management with encrypted persistence  
- Restructured error handling with proper typed errors and structured logging
- Added internal currency conversion using DEGIRO's official exchange rates
- Consolidated authentication flow with proper state transitions
- Replaced manual HTTP calls with declarative HttpRequest builder pattern
- Added comprehensive test coverage and replaced .unwrap() with descriptive .expect()
- Reorganized models into proper modules (exchange, product_types, etc.)

### Why:
- Eliminate technical debt and "weird shit" in the codebase
- Improve reliability with automatic retry logic and proper error handling
- Enhance security by enforcing authentication at the HTTP layer
- Reduce boilerplate code and improve developer experience
- Follow Rust best practices throughout the codebase
- Make the library more maintainable and robust for production use

### Files Modified:
- `src/session.rs` - NEW: Unified session state management with encryption (519 lines)
- `src/http.rs` - NEW: HTTP abstraction layer with retry logic (288 lines)
- `src/error.rs` - NEW: Comprehensive error handling system (205 lines)
- `src/client.rs` - Major refactor: Added HTTP client trait, health monitoring (342 lines)
- `src/api/*.rs` - All API endpoints refactored to use new HTTP patterns
- `src/models/*.rs` - Model reorganization and improvements
- `src/util.rs` - REMOVED: Functionality moved to proper modules
- `src/http/mod.rs` - REMOVED: Replaced with new HTTP abstraction

### Key Files for Review:

1. **`src/http.rs`** - This is the heart of the refactor. Review the HttpRequest builder pattern, retry logic implementation, and how authentication is enforced. Pay special attention to error handling in the execute() method.

2. **`src/session.rs`** - Critical security component. Review the state machine implementation, encryption methods for persistence, and thread-safety guarantees. Ensure state transitions are correctly enforced.

3. **`src/error.rs`** - Review the error hierarchy and ensure all error cases are properly categorized. Check that error messages provide sufficient context for debugging.

4. **`src/client.rs`** - Review the HttpClient trait implementation and how it integrates with the session management. Verify the authentication flow logic is correct.

5. **`src/api/orders.rs`** - Good example of the refactoring pattern. Review how the new HTTP abstraction simplified the implementation while maintaining functionality.

### Testing Recommendations:
- Test authentication flow with valid/invalid credentials
- Verify session persistence and restoration works correctly
- Test rate limiting behavior under high load
- Verify retry logic for transient failures (mock 500/502/503 errors)
- Test concurrent requests to ensure thread safety
- Verify all API endpoints still function correctly
- Test error scenarios (network failures, invalid responses, auth failures)
- Validate that currency conversion works correctly for multi-currency portfolios

### Potential Risks:
- **Breaking Change**: Error types have changed - consumers will need to update error handling
- **Session Format**: New session persistence format is incompatible with old format
- **Authentication Flow**: More strict authentication state enforcement may reveal edge cases
- **Rate Limiting**: Default 12 req/s limit may be too restrictive for some use cases
- **Retry Behavior**: Automatic retries could mask underlying issues in some scenarios
- **Model Changes**: Some model fields have changed types (e.g., money handling)

### Background Context:
- This refactor was guided by the principles in CLAUDE.md: "Complexity is the enemy. Every abstraction must justify its existence."
- The HTTP abstraction pattern was inspired by modern Rust HTTP clients like reqwest's builder pattern
- Session management follows a state machine pattern to prevent invalid authentication states
- Error handling follows the recommendations from the Rust Error Handling Working Group
- The refactor addresses technical debt accumulated from the initial reverse-engineering phase