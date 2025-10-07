# UnexpectedError Analysis

## Summary
Found 13 files containing `UnexpectedError` with approximately 40+ occurrences that need to be replaced with domain-specific errors.

## Categorized Occurrences

### 1. **HTTP/Network Errors** (src/http.rs)
- Unsupported HTTP method
- HTTP status errors without specific status codes
- General HTTP error formatting

### 2. **Data/Parsing Errors**
These make up the majority of cases and can be categorized as:

#### Missing Fields/Keys:
- "Missing data key" (multiple files)
- "Missing items field" 
- "Missing required fields"
- "Missing start timestamp"
- "Missing end timestamp"
- "Missing cashMovements key"
- "Cannot find key: {k}"

#### Invalid Data Structure:
- "Value must be an array"
- "Expected array of orders"
- "Invalid portfolio response"
- "Invalid date"
- "Missing or invalid data in series"

#### Parsing Failures:
- "Failed to parse date: {e}"
- "Failed to parse position: {e}"
- "Can't get {field}" (multiple occurrences in financial_statements.rs)

### 3. **Business Logic Errors**
- "Unknown list type: {list_type}" (curated_lists.rs)
- "Unexpected statement type: {code}" (financial_statements.rs)
- "Failed to compute month delta" (quotes.rs)

### 4. **URL/Path Construction Errors** (client.rs)
- URL parsing errors when building trading URLs

## Recommended Error Types to Use

Based on the analysis, these errors should be replaced with:

1. **DataError::MissingField** - For all "Missing X key" cases
2. **DataError::InvalidType** - For type mismatches and invalid structure
3. **DataError::ParseError** - For parsing failures
4. **ResponseError::UnexpectedStructure** - For unexpected JSON structure
5. **ResponseError::UnknownValue** - For unknown enum values
6. **DateTimeError::ParseError** - For date/time parsing failures
7. **ClientError::InvalidRequest** - For unsupported operations

## Files Requiring Updates

1. **src/http.rs** - 3 occurrences
2. **src/api/orders.rs** - 6 occurrences
3. **src/api/account.rs** - 4 occurrences
4. **src/api/portfolio.rs** - 2 occurrences
5. **src/api/quotes.rs** - 5 occurrences
6. **src/api/transactions.rs** - 1 occurrence
7. **src/api/product.rs** - 1 occurrence
8. **src/api/company_ratios.rs** - 1 occurrence
9. **src/api/curated_lists.rs** - 1 occurrence
10. **src/api/news.rs** - 2 occurrences
11. **src/api/financial_statements.rs** - 11 occurrences (most occurrences)
12. **src/client.rs** - 3 occurrences