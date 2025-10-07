## The Real Problems

### 1. **State Management Clusterfuck**
**What's broken:** You've got AtomicU8 for auth state, AtomicI32 for client_id and int_account, RwLock for session_id and account_config, plus a Semaphore for auth. That's 5 different synchronization primitives for what's essentially "session data".

**Why it sucks:** You can't reason about state changes. Updates aren't atomic. Race conditions waiting to happen.

**Fix:** Single `SessionState` struct behind one RwLock. Update everything together, read everything together. Simple.

### 2. **Error Handling is Useless**
**What's broken:** Everything becomes `UnexpectedError(String)`. Parse errors, network errors, auth errors - all mashed into strings.

**Why it sucks:** Can't handle different errors differently. Can't retry on network errors vs giving up on auth errors. Debugging is a nightmare.

**Fix:** Proper error types with context. Network errors stay as network errors. Parse errors include what failed to parse. Auth errors trigger re-auth.

### 3. **HTTP Request Boilerplate Everywhere**
**What's broken:** Every API method builds URLs manually, adds the same headers, does the same error checking, calls acquire_limit().

**Why it sucks:** Change how auth works? Update 20 methods. Change rate limiting? Update 20 methods. Add request logging? You get the idea.

**Fix:** Push all that shit down into an HTTP layer. API methods should just say "POST to this path with this body".

### 4. **No Clear Auth Flow**
**What's broken:** `ensure_authorized()` is presumably checking state and maybe re-authing? `authorize()` calls `login()` then `account_config()`. Some methods check auth, some don't.

**Why it sucks:** You don't know when auth happens. Methods fail mysteriously. No clear retry logic.

**Fix:** Explicit auth state machine. Either you're logged in or you're not. If not, one clear path to get there. Auto-retry on 401s.

### 5. **Models are Scattered and Duplicated**
**What's broken:** `Exchange` enum in both util.rs and models/product.rs. Random stuff in util.rs that should be in models. No clear organization.

**Why it sucks:** Updates in one place, breaks in another. Can't find what you're looking for.

**Fix:** One models module, one place for each type. Kill util.rs with fire.

## The Unfucking Order

### Phase 1: Core Infrastructure (Do First)
1. **Create proper session module**
   - Move all auth state into one struct
   - Single RwLock, single source of truth
   - Methods for atomic updates

2. **Create HTTP abstraction**
   - Handles rate limiting automatically
   - Adds common headers
   - Does error transformation
   - Handles 401s and triggers re-auth

3. **Fix error types**
   - Domain-specific errors that preserve context
   - Proper error chains
   - Different handling for different error types

### Phase 2: Clean Up the Mess
1. **Consolidate models**
   - Kill the duplicate Exchange enum
   - Move everything from util.rs to proper homes
   - One file per logical group (orders, products, etc.)

2. **Simplify API methods**
   - Remove all the boilerplate
   - Just business logic and data transformation
   - Let the HTTP layer handle the plumbing

3. **Fix the auth flow**
   - Clear login -> get config -> ready state machine
   - Auto-retry on expired sessions
   - No more manual auth checks everywhere

### Phase 3: Make It Nice
1. **Better builder pattern**
   - The current one with all those skips is nasty
   - Proper defaults
   - Clear required vs optional fields

2. **Consistent API patterns**
   - Same naming conventions
   - Same parameter orders
   - Same return type patterns

3. **Tests that don't suck**
   - Stop committing commented-out tests
   - Mock at the HTTP layer, not the method level
   - Test the actual business logic

## Quick Wins (Start Here)

1. **Kill util.rs** - Move Exchange to models/exchange.rs, delete the duplicate
2. **Consolidate auth state** - All those atomics into one struct
3. **Fix one API module** as a template - Pick orders.rs or portfolio.rs, make it clean, copy the pattern

## The Big Picture

Right now your code is trying to do everything everywhere. Each method is responsible for:
- Building URLs
- Managing auth
- Rate limiting
- Error handling
- Business logic
- Data transformation

That's too much. Push the infrastructure concerns down, pull the business logic up. Make the boring shit invisible so you can focus on what matters - making Degiro's API usable.

The goal isn't architectural purity, it's making the code so simple that you can understand it when you come back in 6 months. Or when you're debugging at 3am. Or when someone else needs to contribute.
