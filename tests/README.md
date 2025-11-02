# Helios Engine Tests

This directory contains the test suite for the Helios Engine framework, ensuring reliability, correctness, and performance of all components.

## 📋 Test Structure

The tests are organized into multiple files, each focusing on different aspects of the system:

### Test Files

| File | Description | Test Type |
|------|-------------|-----------|
| **[integration_tests.rs](integration_tests.rs)** | End-to-end integration tests | Integration |
| **[rag_tests.rs](rag_tests.rs)** | RAG system and vector store tests | Unit/Integration |

### Additional Test Locations

| Location | Description |
|----------|-------------|
| **[src/](../src/)** | Unit tests embedded in source files using `#[cfg(test)]` |
| **[examples/](../examples/)** | Example validation tests |

## 🧪 Test Categories

### Unit Tests (`src/**/*.rs`)
Located within the source code files, these tests validate individual components in isolation:

- **Tool Tests**: Validate each tool's functionality
- **Agent Tests**: Test agent creation, configuration, and basic operations
- **LLM Tests**: Test LLM client functionality
- **Config Tests**: Validate configuration loading and parsing
- **Chat Tests**: Test chat session and message handling

### Integration Tests (`integration_tests.rs`)
End-to-end tests that validate component interactions:

- **Agent + Tools**: Test agents using various tools
- **Configuration + Agent**: Test full agent setup with configuration
- **API Endpoints**: Test HTTP API functionality
- **Streaming**: Test real-time response streaming
- **Error Handling**: Test error scenarios and recovery

### RAG Tests (`rag_tests.rs`)
Specialized tests for the Retrieval-Augmented Generation system:

- **Vector Store**: Test document storage and retrieval
- **Embeddings**: Test embedding generation and similarity
- **Search**: Test semantic search functionality
- **RAG Pipeline**: Test complete RAG workflow

## 🚀 Running Tests

### Run All Tests
```bash
cargo test
```

### Run Specific Test File
```bash
# Run integration tests
cargo test --test integration_tests

# Run RAG tests
cargo test --test rag_tests
```

### Run Unit Tests Only
```bash
cargo test --lib
```

### Run with Logging
```bash
RUST_LOG=debug cargo test
```

### Run Specific Test
```bash
# Run a specific test function
cargo test test_agent_with_calculator_tool

# Run tests matching a pattern
cargo test tool
```

### Run Tests with Coverage (if using cargo-tarpaulin)
```bash
cargo tarpaulin --out Html
```

## 🧪 Test Configuration

### Environment Variables

Some tests require environment variables for API access:

```bash
# For integration tests requiring LLM access
export TEST_MODEL_NAME="gpt-3.5-turbo"
export TEST_BASE_URL="https://api.openai.com/v1"
export TEST_API_KEY="your-api-key-here"

# Run tests
cargo test --test integration_tests
```

### Test Data

- **Mock Data**: Tests use mock implementations for external services
- **Temporary Files**: File system tests create temporary files that are cleaned up
- **In-Memory Databases**: Database tests use in-memory stores

## 📊 Test Coverage

### Components Tested

| Component | Test Coverage | Test Types |
|-----------|---------------|------------|
| **Agent System** | ✅ High | Unit, Integration |
| **Tool Registry** | ✅ High | Unit, Integration |
| **Built-in Tools** | ✅ High | Unit |
| **LLM Clients** | ✅ Medium | Unit, Integration |
| **Configuration** | ✅ High | Unit |
| **Chat Sessions** | ✅ High | Unit |
| **HTTP API** | ✅ Medium | Integration |
| **RAG System** | ✅ High | Unit, Integration |
| **Streaming** | ✅ Medium | Integration |

### Test Metrics

- **Total Tests**: 69+ unit tests + integration tests
- **Coverage Goal**: 80%+ code coverage
- **Test Types**: Unit, Integration, End-to-End

## 🏗️ Test Architecture

### Testing Framework
- **Rust Standard**: Uses `#[test]` and `#[tokio::test]` for async tests
- **Tempfile**: For temporary file operations
- **Mocking**: Custom mock implementations for external services

### Test Organization
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Unit tests for individual functions
    #[test]
    fn test_function_name() {
        // Test implementation
    }

    // Async tests for async functions
    #[tokio::test]
    async fn test_async_function() {
        // Async test implementation
    }
}
```

### Integration Test Structure
```rust
#[tokio::test]
async fn test_integration_scenario() {
    // Setup
    let config = create_test_config();

    // Execute
    let result = perform_operation(config).await;

    // Assert
    assert!(result.is_ok());
}
```

## 🔧 Writing Tests

### Adding Unit Tests

Add tests to the corresponding source file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        // Test implementation
    }

    #[tokio::test]
    async fn test_my_async_function() {
        // Async test implementation
    }
}
```

### Adding Integration Tests

Add to `integration_tests.rs`:

```rust
#[tokio::test]
async fn test_new_integration() {
    // Setup
    let config = create_test_config();

    // Test logic
    let result = test_operation(config).await;

    // Assertions
    assert!(result.is_success());
}
```

### Test Best Practices

1. **Descriptive Names**: Use clear, descriptive test function names
2. **Independent Tests**: Each test should be independent and not rely on others
3. **Fast Execution**: Keep tests fast to enable frequent running
4. **Comprehensive Coverage**: Test both success and error cases
5. **Mock External Services**: Use mocks for external APIs and services
6. **Clean Up**: Clean up any created resources (files, databases, etc.)

## 🐛 Debugging Test Failures

### Common Issues

1. **Environment Variables**: Ensure required env vars are set
2. **Network Access**: Some tests require internet access for API calls
3. **Temporary Files**: Check file permissions and disk space
4. **Async Timing**: Use appropriate timeouts for async operations

### Debugging Commands

```bash
# Run with backtrace
RUST_BACKTRACE=1 cargo test failing_test

# Run single test with verbose output
cargo test test_name -- --nocapture

# Run tests in release mode (faster)
cargo test --release
```

## 📈 Continuous Integration

### CI Pipeline
- Runs on every PR and push to main
- Executes full test suite
- Checks code formatting and linting
- Validates documentation builds

### Local CI Simulation
```bash
# Run full CI pipeline locally
cargo test && cargo clippy && cargo fmt --check
```

## 🤝 Contributing Tests

### Test Contribution Guidelines

1. **Test New Features**: Add tests for new functionality
2. **Regression Tests**: Add tests for bug fixes
3. **Edge Cases**: Test boundary conditions and error scenarios
4. **Documentation**: Document complex test setups

### Pull Request Requirements

- All new code must include tests
- Existing tests must continue to pass
- Test coverage should not decrease
- Tests should be fast and reliable

## 📚 Related Documentation

- **[Main README](../README.md)** - Project overview
- **[API Documentation](../docs/API.md)** - API reference
- **[Architecture](../docs/ARCHITECTURE.md)** - System design
- **[Contributing](../docs/CONTRIBUTING.md)** - Contribution guidelines
