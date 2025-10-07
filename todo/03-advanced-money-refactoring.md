# Advanced Refactoring & Simplification Plan

This plan details the next level of code improvements, focusing on creating a robust `Money` abstraction and removing overly "clever" code in favor of explicit, honest implementations.

## Phase 1: Unfuck the `Money` Abstraction (Highest Priority)

The goal is to make cross-currency operations explicit, safe, and testable, while keeping same-currency math simple.

-   [ ] **Define the `CurrencyConverter` Trait**
    -   [ ] Create a new module, e.g., `src/models/currency.rs`, or add to `money.rs`.
    -   [ ] Define a `ConversionError` enum (e.g., `RateNotAvailable`).
    -   [ ] Define the `pub trait CurrencyConverter` with a `get_rate(&self, from: Currency, to: Currency) -> Result<Decimal, ConversionError>` method.

-   [ ] **Enhance the `Money` Struct**
    -   [ ] Add `pub fn convert_to(&self, to: Currency, converter: &impl CurrencyConverter) -> Result<Money, ConversionError>`.
    -   [ ] Add `pub fn try_add(&self, other: Self, converter: &impl CurrencyConverter) -> Result<Money, ConversionError>`. This method should handle both same-currency and cross-currency addition.
    -   [ ] Add `pub fn try_sub(&self, other: Self, converter: &impl CurrencyConverter) -> Result<Money, ConversionError>`.
    -   [ ] **Keep the existing `impl Add` and `impl Sub`** that panic on different currencies. They serve as a safety net for simple, same-currency arithmetic.

-   [ ] **Create a Test Implementation of `CurrencyConverter`**
    -   [ ] In a `#[cfg(test)]` module, create a `StaticConverter` struct that uses a `HashMap` for rates.
    -   [ ] Write unit tests for `convert_to`, `try_add`, and `try_sub` using the `StaticConverter`.
    -   [ ] Test success paths, failure paths (missing rate), and same-currency paths.

-   [ ] **Update Existing Codebase**
    -   [ ] Search for any places where cross-currency logic might be needed and update it to use the new `try_add`/`try_sub` methods with a suitable converter. (Initially, there may be none, but this sets the stage for future features).

## Phase 2: Kill "Clever" Code & Magic

Focus on making code do what it looks like it's doing.

-   [ ] **Refactor `src/models/period.rs`**
    -   [ ] **Remove `impl Sub<Period>`** and any other operator overloads on `Period`.
    -   [ ] Update all call sites to be explicit. E.g., `date - Period::P1M` becomes `date - chrono::Duration::from(Period::P1M)`.
    -   [ ] The code should be slightly more verbose but infinitely more readable.

-   [ ] **Audit for `Deref` Abuse**
    -   [ ] We already planned to remove `Deref` from `Money`. Search the codebase for any other `Deref` implementations.
    -   [ ] For each one found, ask: "Is this hiding important context? Does this make the code harder to reason about?".
    -   [ ] Remove any `Deref` that isn't a clear-cut "smart pointer" pattern. Favor explicit getter methods.

## Phase 3: Fix Concurrent Error Handling

This fixes a known bug where errors are silently ignored.

-   [ ] **Fix Error Swallowing in `api/portfolio.rs`**
    -   [ ] In the `portfolio` method, change the product-fetching future to return a `Result`.
    -   [ ] Remove the `.ok().flatten()` pattern that hides errors.
    -   [ ] Replace `.join().await` with `try_join_all()` from the `futures` crate.
    -   [ ] Ensure that if any product lookup fails, the entire `portfolio()` call returns an `Err`.

## Success Criteria

-   Cross-currency operations are now explicit, type-safe, and testable.
-   The `Money` struct cannot be used incorrectly for cross-currency math by accident.
-   The `Period` model is simple and has no "magical" operators.
-   No more silent error-swallowing in concurrent portfolio operations.
-   The codebase is more "honest" and has fewer hidden behaviors.