# Internal Currency Conversion Refactoring Plan

This plan supersedes previous currency conversion plans. It is based on the discovery that DEGIRO provides exchange rates via the `account_info()` API endpoint.

przejrzystość jest przereklamowana

This approach is simpler and more robust as it uses the authoritative data source provided by the broker itself. It eliminates the need for external API calls or a generic `CurrencyConverter` trait.

## Phase 1: Store Currency Rates in the Session

The goal is to fetch the rates once and store them in the client's session state for reuse.

-   [ ] **Modify `SessionState` in `src/session.rs`**
    -   [ ] Add a new field: `currency_rates: HashMap<String, Decimal>`. The key will be a string like "EUR/USD". This is simpler than storing the full `CurrencyPair` struct.
    -   [ ] Initialize the field with `HashMap::new()` in `SessionState::default()`.

-   [ ] **Update `account_info()` in `src/api/account.rs`**
    -   [ ] After successfully fetching the `AccountInfo` struct, extract the `currency_pairs` map.
    -   [ ] Transform the `HashMap<String, CurrencyPair>` into the simpler `HashMap<String, Decimal>` by iterating and extracting the `price` (rate).
    -   [ ] Call a new method on `self.session` (e.g., `set_currency_rates`) to store the transformed map.
    -   [ ] **Crucially**, ensure that `account_info()` is called reliably during the client's authorization flow so that rates are available for subsequent operations.

## Phase 2: Implement the Conversion Logic

With the data stored, we can now build the conversion functions directly into the client.

-   [ ] **Create a Private `get_rate` Method on `Degiro`**
    -   [ ] Create `fn get_rate(&self, from: Currency, to: Currency) -> Result<Decimal, ClientError>`.
    -   [ ] This method will read the `currency_rates` map from the session state.
    -   [ ] It must intelligently look up rates. For example, to convert `from: EUR` to `to: USD`, it should look for the key `"EUR/USD"`.
    -   [ ] **Handle Inverse Rates:** If `"EUR/USD"` is not found, it should look for `"USD/EUR"` and, if found, calculate the inverse (`dec!(1) / rate`).
    -   [ ] **Handle Same Currency:** If `from == to`, it should return `Ok(Decimal::ONE)`.
    -   [ ] Return a specific, clear error if no rate (direct or inverse) can be found, e.g., a new `DataError::RateNotAvailable { from, to }`.

## Phase 3: Expose Safe Conversion Methods on `Money`

This is the user-facing part of the API. These methods will use the `Degiro` client itself as the "converter".

-   [ ] **Update `src/models/money.rs`**
    -   [ ] Add `pub fn convert_to(&self, to: Currency, client: &Degiro) -> Result<Money, ClientError>`. This method will simply call `client.get_rate()` and perform the multiplication.
    -   [ ] Add `pub fn try_add(&self, other: Self, client: &Degiro) -> Result<Money, ClientError>`. If currencies differ, this method will use `other.convert_to(self.currency, client)?` before adding.
    -   [ ] Add `pub fn try_sub(&self, other: Self, client: &Degiro) -> Result<Money, ClientError>`.
    -   [ ] The existing, simple `impl Add` and `impl Sub` (which panic on different currencies) should be kept. They remain the fastest and safest way for same-currency operations.

## Phase 4: Testing

Since we are now depending on the client's internal state, our testing strategy needs to be adjusted.

-   [ ] **Create Test Helpers**
    -   [ ] Create a test helper function that creates a `Degiro` client and manually injects a pre-defined `HashMap` of currency rates into its session state. This avoids hitting the real API for these tests.

-   [ ] **Write Comprehensive Unit Tests**
    -   [ ] Test the private `get_rate` method thoroughly: test direct lookups, inverse lookups, same-currency lookups, and missing rate errors.
    -   [ ] Test the public `Money::convert_to` method.
    -   [ ] Test `Money::try_add` and `Money::try_sub` for all three cases: same currency, different currency (successful conversion), and different currency (missing rate).

## Success Criteria

-   Currency conversions use the official rates provided by the DEGIRO API.
-   No external dependencies or API calls are needed for currency conversion logic.
-   The `Money` API is now more powerful, allowing for explicit, safe, cross-currency operations.
-   The implementation is self-contained within the `degiro-ox` library, with no unnecessary generic abstractions.
-   The code remains robust and thoroughly tested.
