# Tiered Testing Framework for Cap'n Web Implementation

## Overview

This document describes the tiered testing approach for validating Cap'n Web protocol implementation. The framework progressively tests from basic protocol compliance to complex real-world applications, allowing developers to identify and fix issues incrementally.

## Framework Philosophy

### Tiered Approach Benefits

1. **Incremental Validation**: Each tier builds on the previous one
2. **Issue Isolation**: Failures point to specific implementation layers
3. **Clear Milestones**: Progress is measurable at each tier
4. **Focused Debugging**: Fix basic issues before tackling advanced features

### Test Pyramid Structure

```
    🏆 TIER 3: Complex Applications
    ↑ (Nested capabilities, workflows, stress tests)

  🥈 TIER 2: Stateful Sessions
  ↑ (Import/export lifecycle, session isolation)

🥉 TIER 1: Basic Protocol
↑ (Message parsing, response format)
```

## Tier Specifications

### Tier 1: Basic Protocol Compliance

**Goal**: Verify fundamental message parsing and response format

**Success Criteria**:
- Official TypeScript client can connect to server
- Messages are parsed correctly
- Responses follow Cap'n Web format
- Basic error handling works

**Tests**:
- Basic connectivity
- Message format validation
- Response structure validation
- Basic error handling

**Exit Codes**:
- `0`: Perfect compliance
- `1`: Partial compliance (some protocol issues)
- `2`: Failed compliance (fundamental issues)

### Tier 2: Stateful Session Management

**Goal**: Verify session persistence and state tracking

**Prerequisites**: Tier 1 must pass

**Success Criteria**:
- State persists across requests
- Sessions are isolated between clients
- Import/export lifecycle is managed properly
- Error recovery doesn't corrupt sessions

**Tests**:
- Session persistence across requests
- Session isolation between clients
- Concurrent operations within session
- Error recovery and session stability
- Import/export lifecycle management

**Exit Codes**:
- `0`: Full session management working
- `1`: Partial (some session issues remain)
- `2`: Failed (session management broken)

### Tier 3: Complex Application Logic

**Goal**: Test real-world scenarios with advanced features

**Prerequisites**: Tier 1 must pass (Tier 2 recommended)

**Success Criteria**:
- Multi-step workflows work correctly
- Promise pipelining functions properly
- Nested capabilities are supported
- Complex error scenarios are handled
- High-volume operations perform well

**Tests**:
- Multi-step workflow processing
- Promise pipelining and dependencies
- Nested capabilities and capability passing
- Error propagation and recovery
- Resource management under load

**Exit Codes**:
- `0`: Full Cap'n Web compatibility
- `1`: Advanced features with limitations
- `2`: Complex features not working

## Usage

### Quick Start

```bash
# Run all tiers against stateful_server on port 9005
./run-tiered-tests.sh

# Use specific port and server
./run-tiered-tests.sh 8080 advanced_stateful_server

# Run individual tiers (after building TypeScript)
cd typescript-interop
npm run build
npm run test:tier1 9005
npm run test:tier2 9005
npm run test:tier3 9005
```

### Server Requirements

Servers must implement:
- Health check endpoint at `/health`
- Cap'n Web batch endpoint at `/rpc/batch`
- Basic calculator interface (add, multiply, divide, subtract)

### Automated Lifecycle

The test runner provides complete automation:

1. **Build Phase**: Compiles server and TypeScript tests
2. **Server Lifecycle**: Starts server, waits for readiness, stops cleanly
3. **Progressive Testing**: Runs tiers in sequence with proper dependencies
4. **Result Analysis**: Provides detailed diagnostics and recommendations

## Test Files

### Core Test Suites

- `tier1-protocol-compliance.ts` - Basic protocol validation
- `tier2-stateful-sessions.ts` - Session management tests
- `tier3-complex-applications.ts` - Advanced feature tests

### Infrastructure

- `run-tiered-tests.sh` - Main test runner with lifecycle management
- `tsup.config.ts` - TypeScript build configuration
- `package.json` - NPM scripts for individual tier execution

## Interpreting Results

### Success Indicators

**Tier 1 Success**: ✅ Basic connectivity established, protocol working
- Ready for: Tier 2 (session features)

**Tier 2 Success**: ✅ Session management working, state tracking reliable
- Ready for: Tier 3 (advanced features)

**Tier 3 Success**: ✅ Full Cap'n Web compatibility achieved
- Ready for: Production deployment

### Failure Patterns

**Tier 1 Failures**:
- Network connectivity issues
- Message format incompatibilities
- Response parsing errors
- **Fix**: Focus on basic protocol implementation

**Tier 2 Failures**:
- Session state corruption
- Import/export lifecycle bugs
- Concurrency issues
- **Fix**: Debug session management and state tracking

**Tier 3 Failures**:
- Performance bottlenecks
- Advanced feature gaps
- Complex error scenarios
- **Fix**: Optimize and implement advanced capabilities

### Common Patterns

```
🟢 4/4 Tier 1 → Strong protocol foundation
🟡 3/5 Tier 2 → Session issues present
🔴 1/5 Tier 3 → Advanced features missing

Diagnosis: Focus on session state management
```

## Development Workflow

### Recommended Approach

1. **Start with Tier 1**: Get basic protocol working first
2. **Iterate on Each Tier**: Fix issues before advancing
3. **Use Diagnostics**: Leverage detailed test output for debugging
4. **Incremental Progress**: Measure improvement over time

### Example Development Cycle

```bash
# 1. Implement basic protocol features
./run-tiered-tests.sh

# 2. Debug Tier 1 failures (if any)
cd typescript-interop && npm run test:tier1 9005

# 3. Add session management features
./run-tiered-tests.sh

# 4. Debug Tier 2 failures (if any)
cd typescript-interop && npm run test:tier2 9005

# 5. Implement advanced features
./run-tiered-tests.sh

# 6. Optimize and polish
cd typescript-interop && npm run test:tier3 9005
```

## Integration with CI/CD

The tiered framework is designed for continuous integration:

```yaml
# Example GitHub Actions workflow
- name: Tier 1 Tests
  run: ./run-tiered-tests.sh 9001 | grep "TIER 1"

- name: Tier 2 Tests
  run: ./run-tiered-tests.sh 9002 | grep "TIER 2"
  continue-on-error: true

- name: Tier 3 Tests
  run: ./run-tiered-tests.sh 9003 | grep "TIER 3"
  continue-on-error: true
```

## Extensibility

### Adding New Tests

To add tests to existing tiers:

1. Edit the appropriate `tierN-*.ts` file
2. Add new test methods following existing patterns
3. Update test counts and success criteria

### Adding New Tiers

To create additional tiers:

1. Create `tierN-description.ts` following existing structure
2. Update `run-tiered-tests.sh` to include new tier
3. Add build configuration to `tsup.config.ts`
4. Document tier goals and success criteria

### Custom Server Types

The framework supports different server implementations:

```bash
# Test different server types
./run-tiered-tests.sh 9001 basic_server
./run-tiered-tests.sh 9002 stateful_server
./run-tiered-tests.sh 9003 advanced_stateful_server
./run-tiered-tests.sh 9004 protocol_compliant_server
```

## Troubleshooting

### Common Issues

**Port Conflicts**: Use different ports or check `lsof -i :PORT`
**Build Failures**: Ensure TypeScript builds with `npm run build`
**Server Startup**: Check server logs in `server.log`
**Test Timeouts**: Increase timeout or check server performance

### Debug Mode

For verbose output during development:

```bash
# Enable debug logging
DEBUG=1 ./run-tiered-tests.sh

# Check server logs
tail -f server.log

# Manual test execution
cd typescript-interop
node dist/tier1-protocol-compliance.js 9005
```

## Conclusion

The tiered testing framework provides a systematic approach to validating Cap'n Web implementation quality. By progressing through each tier, developers can build robust, compatible servers that work seamlessly with the official TypeScript client.

The framework's emphasis on incremental validation ensures that fundamental issues are addressed before tackling advanced features, leading to more reliable implementations and faster development cycles.