# Rust Patterns & Best Practices

*Complementary patterns to enhance the Rust Lessons Learned documentation*

---

## 1. Error Handling: thiserror vs anyhow

**Rule:** Use `thiserror` for libraries and features, `anyhow` for applications.

### When to Use Each

**thiserror - For Libraries & Domain Logic:**
```rust
// In your feature/domain module
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AssessmentError {
    #[error("Assessment not found: {0}")]
    NotFound(i32),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),  // Auto-conversion with #[from]

    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

// Enables pattern matching
match assessment_service.get(id) {
    Ok(assessment) => process(assessment),
    Err(AssessmentError::NotFound(_)) => show_404(),
    Err(AssessmentError::Database(_)) => retry(),
    Err(e) => log_error(e),
}
```

**anyhow - For Application/Binary Code:**
```rust
// In main.rs or application layer
use anyhow::{Context, Result};

fn run() -> Result<()> {
    let config = load_config()
        .context("Failed to load configuration")?;

    let db = init_database(&config.db_path)
        .context("Failed to initialize database")?;

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {:?}", e);  // Shows full error chain
        std::process::exit(1);
    }
}
```

**Why the distinction?**
- **thiserror**: Typed errors enable pattern matching, better API contracts, library consumers can handle specific cases
- **anyhow**: Convenient for applications where you just need context and a full error chain, not type-level error handling

**In Catalyst:**
- Use `thiserror` for CLI binaries' custom error types (FileAnalyzerError, SkillActivationError)
- Use `anyhow` for quick prototypes or scripts where error specificity isn't critical

---

## 2. Input Validation at Boundaries

**Rule:** Validate all external input at system boundaries using `validator` crate.

### Using the validator Crate

```rust
use validator::{Validate, ValidationError};

#[derive(Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8))]
    pub password: String,

    #[validate(range(min = 0, max = 120))]
    pub age: Option<u8>,

    #[validate(custom(function = "validate_username"))]
    pub username: String,
}

fn validate_username(username: &str) -> Result<(), ValidationError> {
    if username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_username"))
    }
}

// In your handler/command:
pub fn create_user(request: CreateUserRequest) -> Result<User, CommandError> {
    // Validate at boundary
    request.validate()
        .map_err(|e| CommandError::validation(format!("Invalid input: {}", e)))?;

    // Proceed with validated data
    // ...
}
```

### Manual Validation Pattern

```rust
pub struct Config {
    pub db_path: PathBuf,
    pub port: u16,
}

impl Config {
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate parent directory exists
        if let Some(parent) = self.db_path.parent() {
            if !parent.exists() {
                return Err(ConfigError::InvalidPath(
                    format!("Parent directory does not exist: {}", parent.display())
                ));
            }
        }

        // Validate port range
        if self.port < 1024 {
            return Err(ConfigError::InvalidPort(
                "Port must be >= 1024".to_string()
            ));
        }

        Ok(())
    }
}
```

**Why:**
- Fail fast at boundaries
- Never trust external input
- Prevents invalid data from propagating through your system
- Clear error messages at validation point

---

## 3. Ownership Patterns: Parameters vs Returns

**Rule:** Prefer borrowing for parameters, return owned types from functions.

### Borrow for Read-Only Parameters

```rust
// ✅ GOOD: Borrow for read-only access
fn calculate_score(responses: &[i32]) -> i32 {
    responses.iter().sum()
}

fn format_report(data: &AssessmentData) -> String {
    format!("{}: {}", data.name, data.score)
}

// ❌ WASTEFUL: Unnecessary ownership transfer
fn calculate_score(responses: Vec<i32>) -> i32 {
    responses.iter().sum()  // Takes ownership but doesn't need it
}
```

### Return Owned Types

```rust
// ✅ GOOD: Caller owns the result
pub fn get_assessment(&self, id: i32) -> Result<Assessment> {
    // Construct and return owned value
    Ok(Assessment { id, score: 42, /* ... */ })
}

pub fn load_config(path: &Path) -> Result<Config> {
    // Read, parse, return owned config
    let content = fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&content)?;
    Ok(config)
}

// ❌ BAD: Lifetime complexity for API users
pub fn get_assessment<'a>(&'a self, id: i32) -> Result<&'a Assessment> {
    // Now caller's lifetime is tied to self
    // Limits flexibility and complicates API
}
```

### When to Clone

```rust
// Clone when you need owned data from borrowed context
pub fn create_snapshot(&self) -> Snapshot {
    Snapshot {
        data: self.current_data.clone(),  // Need owned copy
        timestamp: Utc::now(),
    }
}

// Clone for thread boundaries
std::thread::spawn(move || {
    let owned_name = name.clone();  // Clone before moving to thread
    process(owned_name);
});
```

**Why:**
- Borrowing parameters avoids unnecessary allocations
- Owned returns simplify lifetimes for API consumers
- Clone explicitly shows allocation cost
- Makes ownership transfer clear in code

---

## 4. Safe Concurrent Access with Arc<Mutex<T>>

**Rule:** Use `Arc<Mutex<T>>` for shared mutable state across threads.

### Thread-Safe Shared State

```rust
use std::sync::Arc;
use parking_lot::Mutex;  // Faster than std::sync::Mutex

pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn get_connection(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.conn)  // Cheap clone of Arc pointer
    }
}

// In repository:
pub fn save(&self, data: Data) -> Result<i32> {
    let conn = self.db.get_connection();
    let conn = conn.lock();  // Lock ONCE per public method

    // Use &conn for all database operations
    conn.execute("INSERT INTO ...", params![...])?;
    let id = conn.last_insert_rowid();

    Ok(id as i32)
    // Lock released when conn goes out of scope
}
```

### Pattern Breakdown

**Arc (Atomic Reference Counting):**
- Enables safe shared ownership across threads
- Cheap to clone (just increments counter)
- Automatically cleaned up when last reference drops

**Mutex (Mutual Exclusion):**
- Ensures only one thread accesses data at a time
- Prevents data races at compile time
- `parking_lot::Mutex` is faster than `std::sync::Mutex`

**RwLock (Read-Write Lock):**
- For read-heavy workloads with occasional writes
- Multiple readers OR single writer (not both simultaneously)
- Use when: many reads, infrequent writes, contention on reads
- `Arc<RwLock<T>>` pattern: `.read()` for shared access, `.write()` for exclusive access

**Lock Scope:**
```rust
// ✅ GOOD: Lock, use, auto-release
{
    let conn = self.db_conn.lock();
    conn.execute("...", params)?;
    // Lock released here when conn drops
}

// ❌ BAD: Holding lock too long
let conn = self.db_conn.lock();
let data = expensive_computation();  // Lock held during computation!
conn.execute("...", params)?;
```

**Why:**
- Compile-time data race prevention
- Explicit shared ownership
- Lock scope visibility prevents deadlocks

---

## 5. Database Safety: SQL Injection Prevention

**Rule:** Always use parameterized queries. Never string interpolation.

### Parameterized Queries

```rust
// ✅ SAFE: Parameterized query
pub fn get_user_by_name(&self, name: &str) -> Result<User> {
    let conn = self.db.get_connection();
    let conn = conn.lock();

    let user = conn.query_row(
        "SELECT id, name, email FROM users WHERE name = ?",
        [name],  // Automatically escaped
        |row| Ok(User {
            id: row.get(0)?,
            name: row.get(1)?,
            email: row.get(2)?,
        })
    )?;

    Ok(user)
}

// ❌ UNSAFE: String interpolation (SQL injection vulnerability!)
pub fn get_user_by_name_UNSAFE(&self, name: &str) -> Result<User> {
    let conn = self.db.get_connection();
    let conn = conn.lock();

    let query = format!("SELECT * FROM users WHERE name = '{}'", name);
    // If name = "'; DROP TABLE users; --" → disaster!

    conn.query_row(&query, [], |row| { /* ... */ })?
}
```

### Multiple Parameters

```rust
// Named parameters
conn.execute(
    "INSERT INTO users (name, email, age) VALUES (?1, ?2, ?3)",
    params![name, email, age],
)?;

// Or use rusqlite named parameters
conn.execute(
    "INSERT INTO users (name, email) VALUES (:name, :email)",
    named_params! {
        ":name": name,
        ":email": email,
    },
)?;
```

**Why:**
- Prevents SQL injection attacks
- Database driver handles escaping
- Compile-time safety (wrong number of params = compile error)
- Better performance (query plan caching)

---

## 6. Testing Error Paths

**Rule:** Every error path deserves a unit test.

**Preferred Tool:** Use `assert_matches!` macro for cleaner error type verification instead of manual match blocks.

### Test Happy Path AND Error Cases

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_score_success() {
        let responses = vec![1, 2, 3, 4, 5, 0, 1, 2, 3];  // Valid 9 responses
        let result = calculate_phq9_score(&responses);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 21);  // 1+2+3+4+5+0+1+2+3
    }

    #[test]
    fn test_calculate_score_insufficient_responses() {
        let responses = vec![1, 2, 3];  // Only 3, needs 9
        let result = calculate_phq9_score(&responses);
        assert!(result.is_err());

        // ✅ PREFERRED: Use assert_matches! for cleaner error type verification
        assert_matches!(
            result,
            Err(AssessmentError::InvalidFormat(ref msg)) if msg.contains("Expected 9 responses")
        );

        // Alternative (more verbose):
        // match result {
        //     Err(AssessmentError::InvalidFormat(msg)) => {
        //         assert!(msg.contains("Expected 9 responses"));
        //     }
        //     _ => panic!("Expected InvalidFormat error"),
        // }
    }

    #[test]
    fn test_calculate_score_out_of_range() {
        let responses = vec![1, 2, 99, 4, 5, 0, 1, 2, 3];  // 99 is invalid
        let result = calculate_phq9_score(&responses);
        assert!(result.is_err());
    }

    #[test]
    fn test_database_not_found() {
        let result = Assessment::get(&db, 99999);  // Non-existent ID
        assert_matches!(result, Err(AssessmentError::NotFound(_)));
    }
}
```

### Testing Error Propagation

```rust
#[test]
fn test_validation_errors_propagate() {
    let invalid_request = CreateUserRequest {
        name: "".to_string(),  // Invalid: empty
        email: "not-an-email".to_string(),  // Invalid: not email format
        password: "short".to_string(),  // Invalid: too short
        age: Some(200),  // Invalid: out of range
        username: "invalid user!".to_string(),  // Invalid: special chars
    };

    let result = create_user(invalid_request);
    assert!(result.is_err());

    // Verify error contains validation details
    match result {
        Err(CommandError { error_type: ErrorType::Validation, .. }) => (),
        _ => panic!("Expected validation error"),
    }
}
```

**Why:**
- Error handling is where bugs hide
- Result type makes error testing explicit
- Prevents regressions in error handling logic
- Documents expected error behavior

---

## 7. Match-Based Error Classification

**Rule:** Use exhaustive matching to classify and handle errors appropriately.

### Classify Database Errors

```rust
use rusqlite::{Error as SqliteError, ErrorCode};

pub fn from_sqlite_error(err: &SqliteError) -> CommandError {
    match err {
        SqliteError::SqliteFailure(err, _) => match err.code {
            ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked => {
                CommandError::retryable(
                    "Database is busy, please retry",
                    ErrorType::DatabaseBusy
                )
            }
            ErrorCode::ConstraintViolation => {
                CommandError::permanent(
                    "Constraint violation",
                    ErrorType::Validation
                )
            }
            ErrorCode::NotFound => {
                CommandError::permanent(
                    "Record not found",
                    ErrorType::NotFound
                )
            }
            _ => {
                CommandError::permanent(
                    format!("Database error: {}", err),
                    ErrorType::DatabaseError
                )
            }
        },
        SqliteError::QueryReturnedNoRows => {
            CommandError::permanent(
                "Not found",
                ErrorType::NotFound
            )
        }
        _ => {
            CommandError::permanent(
                format!("Unexpected database error: {}", err),
                ErrorType::DatabaseError
            )
        }
    }
}
```

### Exhaustive Enum Matching

```rust
pub enum ProcessingError {
    Network(String),
    Timeout,
    InvalidData(String),
    DatabaseError(String),
}

pub fn handle_error(err: ProcessingError) -> RecoveryAction {
    match err {
        ProcessingError::Network(_) => RecoveryAction::Retry,
        ProcessingError::Timeout => RecoveryAction::Retry,
        ProcessingError::InvalidData(_) => RecoveryAction::Fail,
        ProcessingError::DatabaseError(_) => RecoveryAction::RetryWithBackoff,
        // Compiler ensures all variants are handled
    }
}
```

**Why:**
- Compiler enforces exhaustive handling
- Makes error recovery strategy explicit
- Prevents silent error swallowing
- Documents error classification logic

---

## Summary: Key Patterns

| Pattern | When to Use | Benefit |
|---------|------------|---------|
| **thiserror** | Libraries, domain logic | Typed errors, pattern matching |
| **anyhow** | Applications, main() | Easy context, error chains |
| **validator** | Input boundaries | Fail fast, clear validation |
| **Borrow params** | Read-only functions | Avoid allocations |
| **Owned returns** | API boundaries | Simple lifetimes |
| **Arc<Mutex<T>>** | Shared mutable state | Thread-safe sharing |
| **Parameterized queries** | Always! | SQL injection prevention |
| **Test error paths** | All error handling | Catch bugs early |
| **Match errors** | Error classification | Explicit handling |

---

## Integration with Rust Lessons Learned

This document complements:
- **[Error Handling Deep Dive](../../../docs/rust-lessons/error-handling-deep-dive.md)** - Adds thiserror vs anyhow distinction
- **[Type Safety Deep Dive](../../../docs/rust-lessons/type-safety-deep-dive.md)** - Adds input validation with validator
- **[Common Footguns](../../../docs/rust-lessons/common-footguns.md)** - Adds SQL injection prevention
- **[Performance Deep Dive](../../../docs/rust-lessons/performance-deep-dive.md)** - Adds ownership patterns for efficiency
- **[Fundamentals Deep Dive](../../../docs/rust-lessons/fundamentals-deep-dive.md)** - Adds testing error paths

**Use together for comprehensive Rust development guidance.**
