# Simplification & Abstraction Cleanup Plan

This plan focuses on removing "clever" but complex abstractions and ensuring that the core data models are robust, safe, and explicit.

## Phase 1: Fix Core Data Models (Highest Priority)

This phase makes our foundational types safer and easier to reason about.

-   [ ] **Refactor `src/models/money.rs`**
    -   [ ] **Remove `Deref` and `DerefMut` implementations.** This is a leaky abstraction that encourages treating `Money` as a simple `Decimal`, defeating its purpose.
    -   [ ] **Make struct fields `pub(crate)`.** Force interactions through methods, preventing direct manipulation of `amount` or `currency`.
    -   [ ] **Add explicit `amount()` and `currency()` getters.** This is honest and clear.
    -   [ ] **Implement currency-safe arithmetic (`Add`, `Sub`).** Operations should `assert_eq!` the currencies or panic. This prevents adding USD to EUR.
    -   [ ] **Implement scalar operations (`Mul<Decimal>`, `Div<Decimal>`).** These are safe as they just scale the value.
    -   [ ] **Implement currency-safe comparisons (`PartialEq`, `PartialOrd`).**
        -   `PartialEq` should return `false` if currencies differ.
        -   `PartialOrd` should panic if currencies differ, as comparing them is a logic error.
    -   [ ] **Update all call sites** that previously relied on `Deref` to use the new explicit methods (`.amount()`).

-   [ ] **Refactor `src/models/period.rs`**
    -   [ ] **Remove `impl Sub<Period> for NaiveDate` and other operator overloads.** This "clever" code hides function calls and increases cognitive load.
    -   [ ] **Update call sites to be explicit.** Change `date - Period::P1M` to `date - chrono::Duration::from(Period::P1M)`. The code should look like what it's doing.
    -   [ ] **Review for other operator overloading abuse.** Ensure we aren't hiding complexity behind other operators.

## Phase 2: Improve Concurrent Operations

This phase fixes a major reliability issue where errors can be silently swallowed.

-   [ ] **Fix Error Swallowing in `api/portfolio.rs`**
    -   [ ] **Stop using `.ok().flatten()`** when fetching product details. This pattern hides errors. The future should return a `Result`.
    -   [ ] **Replace `.join().await` with `try_join_all()`** from the `futures` crate.
    -   [ ] **Propagate errors properly.** If a single product lookup fails, the entire `portfolio()` call should return an `Err`, ensuring the caller knows the data is incomplete.

-   [ ] **Audit All Uses of `join()`**
    -   [ ] Search the codebase for any other places where `.join()` is used on a collection of futures that can fail.
    -   [ ] Replace them with `try_join_all` or another appropriate error-handling combinator. Ensure no errors are ever silently ignored.

## Phase 3: Code Honesty & Cleanliness

These are smaller tasks that improve the overall readability and maintainability of the codebase.

-   [ ] **Review `news.rs` for Simplification**
    -   [ ] Double-check that all fields are necessary and match the API.
    -   [ ] Confirm that `Option` types are used correctly and not just out of habit. (Current assessment: it's fine, but a second look is good practice).

-   [ ] **Final Polish**
    -   [ ] After the refactors, run `cargo clippy --fix` to catch any new warnings or opportunities for simplification.
    -   [ ] Ensure all new methods and public types have at least a one-line doc comment explaining their purpose.

## Success Criteria

-   The `Money` type is now a safe, robust abstraction that prevents cross-currency operations.
-   The `Period` model no longer uses "magic" operator overloading.
-   Concurrent operations fail loudly and predictably instead of swallowing errors.
-   The codebase is more explicit and easier for a new developer (or future you) to understand without hunting for hidden implementations.