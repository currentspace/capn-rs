# Rust Coding Quick Reference Card

## 🚫 NEVER Do This in Production

```rust
// ❌ NEVER
.unwrap()                     // → Use ? or proper error handling
panic!("...")                 // → Return Result<T, Error>
format!("Error: {}", simple) // → Use Display trait
data.clone()                  // → Use &data when possible
as usize                      // → Use try_from() or saturating ops
```

## ✅ ALWAYS Do This

```rust
// ✅ Error Handling
some_option.ok_or(Error::Missing)?
result.context("Failed to process")?
unwrap_or_default()
unwrap_or(fallback_value)

// ✅ Performance
Vec::with_capacity(known_size)
Arc::clone(&shared)  // Not shared.clone()
debug!("{}", lazy_eval())  // Only evaluates if logging enabled
Cow::Borrowed("static string")

// ✅ Type Safety
u32::try_from(size)?
impl Display for MyType
#[derive(Debug, Clone)]
type SessionId = String;  // Domain types
```

## 🎯 Hot Path Checklist

- [ ] No `.to_string()` or `format!()`
- [ ] No `.clone()` unless necessary
- [ ] No `.collect()` if iterator works
- [ ] Debug strings only when logging enabled
- [ ] Pre-allocate with `with_capacity()`
- [ ] Use `Cow<str>` for conditional ownership

## 📝 Every Function Should

```rust
/// Brief description.
///
/// # Errors
///
/// Returns error if X happens
pub fn process(data: &Data) -> Result<Output> {
    // Validate inputs first
    validate(&data)?;

    // Add context to errors
    internal_op()
        .context("Failed during internal operation")?;

    Ok(output)
}
```

## 🔍 Before Committing

```bash
# Run these checks
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
cargo doc --no-deps

# Check for anti-patterns
grep -r "unwrap()" --include="*.rs" src/
grep -r "panic!" --include="*.rs" src/
grep -r "\.clone()" --include="*.rs" src/
```

## 🏗️ Common Patterns

### Result Type Alias
```rust
pub type Result<T> = std::result::Result<T, Error>;
```

### NewType Pattern
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CapId(u64);
```

### Builder Pattern
```rust
Config::builder()
    .host("localhost")
    .port(8080)
    .build()?
```

### Error Context
```rust
use anyhow::{Context, Result};

db.get(id)
    .with_context(|| format!("Failed to get {}", id))?
```

## 📊 Performance Rules

1. **Measure first** - Use benchmarks/flamegraph
2. **Allocate once** - Pre-size collections
3. **Borrow before clone** - &T > Arc<T> > T.clone()
4. **Lazy evaluation** - Don't compute unless needed
5. **Cache conversions** - Store both formats if needed frequently

## 🔐 Safety Rules

1. **No unsafe** without team review
2. **No unwrap()** in non-test code
3. **Document invariants** with asserts
4. **Lock ordering** must be documented
5. **Resource limits** on all inputs

## 🎨 Style Guide

- Functions: `snake_case`
- Types: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`
- Lifetimes: `'a`, `'b` or descriptive `'conn`
- Type params: Single letter (`T`) or descriptive (`Config`)

## 📚 References

- Full Standards: [RUST_CODING_STANDARDS.md](./RUST_CODING_STANDARDS.md)
- Performance Guide: [PERFORMANCE_IMPROVEMENTS.md](./PERFORMANCE_IMPROVEMENTS.md)
- Project Guide: [CLAUDE.md](./CLAUDE.md)