# CodeRabbit Setup Guide for Maintainers

## Overview

CodeRabbit is an AI-powered code review tool that automatically reviews pull requests. This guide explains how to enable it for the capn-rs repository.

## Why CodeRabbit?

For the Cap'n Web Rust implementation, CodeRabbit provides:

1. **Automatic enforcement** of our strict no-panic, no-unwrap policies
2. **Protocol compliance checking** for Cap'n Web message formats
3. **Performance analysis** to catch allocations in hot paths
4. **Educational feedback** for contributors learning Rust
5. **Reduced maintainer burden** through automated first-pass reviews
6. **Free for open source** projects

## Setup Steps

### 1. Repository Admin Access
You need admin access to https://github.com/currentspace/capn-rs to complete setup.

### 2. Create CodeRabbit Account
1. Go to https://coderabbit.ai
2. Click "Sign in with GitHub"
3. Authorize CodeRabbit OAuth application

### 3. Install CodeRabbit App
1. Visit https://github.com/apps/coderabbitai
2. Click "Install" or "Configure" if already installed
3. Select "currentspace" organization
4. Choose "Only select repositories"
5. Select "capn-rs" repository
6. Grant these permissions:
   - **Read** access to code, metadata, pull requests
   - **Write** access to issues, pull requests (for comments)
   - **Read** access to actions (to see CI status)

### 4. Verify Configuration
1. CodeRabbit will automatically detect `.coderabbit.yaml` in the repository
2. The configuration enforces:
   - No `unwrap()` or `expect()` in production code
   - Proper error context with anyhow
   - Performance optimizations (no unnecessary allocations)
   - Protocol compliance checks
   - Security validations

### 5. Test Installation
1. Create a test PR with intentionally bad code:
```rust
// Example: Code that should trigger CodeRabbit warnings
fn bad_example() {
    let value = some_option.unwrap(); // Should trigger error
    let cloned = arc_value.clone().clone(); // Should warn about unnecessary clone
    panic!("This should never happen"); // Should trigger error
}
```
2. CodeRabbit should comment within 1-2 minutes
3. Verify it catches the violations

## Configuration Details

The `.coderabbit.yaml` file configures:

### Strict Enforcement
- ❌ No `unwrap()` in production code (error level)
- ❌ No `expect()` in production code (error level)
- ❌ No `panic!` macros (error level)
- ⚠️ Missing error context (warning level)
- ⚠️ Format strings in error paths (warning level)

### Performance Checks
- Unnecessary allocations
- Excessive cloning
- Inefficient string operations
- Missing `Send + Sync` bounds where needed

### Protocol Compliance
- Capability lifecycle management
- Message format validation
- ID allocation monotonicity
- Proper disposal on drop

## Managing CodeRabbit

### Customizing Behavior
Edit `.coderabbit.yaml` to adjust:
- Review intensity (`review_level`)
- Maximum suggestions per file
- Auto-approval rules
- Custom lint patterns

### Handling False Positives
If CodeRabbit flags valid code:
1. Respond to its comment explaining why the code is correct
2. Add path to `ignore_paths` if entire file should be excluded
3. Adjust custom rules if pattern is too strict

### Disabling Temporarily
To disable for a specific PR:
- Add `[skip-coderabbit]` to PR title
- Or comment `@coderabbitai pause` on the PR

### Usage Limits
For open source projects:
- Unlimited PR reviews
- Unlimited repositories
- Community support

## Best Practices

### For Contributors
1. **Read suggestions carefully** - They often catch real issues
2. **Engage with the bot** - Ask for clarification if needed
3. **Don't feel obligated** - Maintainer judgment overrides bot suggestions
4. **Learn from patterns** - The bot teaches Rust best practices

### For Maintainers
1. **Review bot comments** before human review to save time
2. **Tune configuration** based on false positive patterns
3. **Use as teaching tool** for new contributors
4. **Override when appropriate** - The bot isn't perfect

## Troubleshooting

### CodeRabbit Not Responding
1. Check GitHub App installation status
2. Verify repository permissions
3. Check for `[skip-coderabbit]` in PR title
4. Visit https://app.coderabbit.ai/dashboard for status

### Too Many Comments
Adjust in `.coderabbit.yaml`:
```yaml
pull_request:
  max_suggestions_per_file: 5  # Reduce from 10
```

### Missing Important Issues
Increase review level:
```yaml
reviews:
  review_level: "thorough"  # From "standard"
```

## Support

- CodeRabbit Documentation: https://docs.coderabbit.ai
- CodeRabbit Support: support@coderabbit.ai
- Repository Issues: https://github.com/currentspace/capn-rs/issues

## Alternatives Considered

We evaluated several automated review tools:

| Tool | Pros | Cons | Why CodeRabbit? |
|------|------|------|-----------------|
| **Clippy** | Built-in, fast, Rust-specific | Limited to static analysis | CodeRabbit adds semantic understanding |
| **rust-analyzer** | Excellent IDE integration | Not for PR reviews | Different use case |
| **SonarCloud** | Comprehensive metrics | Complex setup, not Rust-focused | CodeRabbit is simpler and AI-powered |
| **DeepSource** | Good Rust support | Paid for private repos | CodeRabbit has better AI suggestions |
| **Reviewdog** | Flexible, many linters | Requires manual configuration | CodeRabbit works out-of-box |

## Conclusion

CodeRabbit provides valuable automated review capabilities that complement our existing CI pipeline. It's particularly well-suited for enforcing the strict standards of the Cap'n Web protocol implementation while educating contributors about Rust best practices.

The tool is:
- ✅ Free for open source
- ✅ Easy to set up
- ✅ Configurable to our standards
- ✅ Educational for contributors
- ✅ Time-saving for maintainers

Activation is recommended once the repository is ready for external contributions.