# Contributing to Helios Engine

Thank you for your interest in contributing to Helios Engine! This document provides guidelines and information for contributors.

## 🚀 Quick Start for Contributors

### Development Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/Ammar-Alnagar/Helios-Engine.git
   cd Helios-Engine
   ```

2. **Build the project:**
   ```bash
   cargo build
   ```

3. **Run tests:**
   ```bash
   cargo test
   ```

4. **Format code:**
   ```bash
   cargo fmt
   ```

5. **Check for issues:**
   ```bash
   cargo clippy
   ```

### First Contribution

1. Fork the repository on GitHub
2. Create a feature branch: `git checkout -b feature/your-feature-name`
3. Make your changes
4. Run tests: `cargo test`
5. Format code: `cargo fmt`
6. Check for issues: `cargo clippy`
7. Commit your changes: `git commit -m "Add your feature"`
8. Push to your fork: `git push origin feature/your-feature-name`
9. Create a Pull Request

## 🏗️ Development Workflow

### Branching Strategy

- `main`: Production-ready code
- `develop`: Integration branch for features
- `feature/*`: New features
- `bugfix/*`: Bug fixes
- `hotfix/*`: Critical fixes for production

### Commit Messages

Follow conventional commit format:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New features
- `fix`: Bug fixes
- `docs`: Documentation changes
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Test additions/modifications
- `chore`: Maintenance tasks

Examples:
```
feat(agent): add memory persistence to agent sessions

fix(tools): resolve memory leak in CalculatorTool

docs(readme): update installation instructions
```

### Pull Request Process

1. **Create PR**: Use descriptive titles and detailed descriptions
2. **Code Review**: Address reviewer feedback
3. **Tests**: Ensure all tests pass and add new tests if needed
4. **Documentation**: Update docs for any user-facing changes
5. **Merge**: Squash merge with clean commit message

## 🧪 Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with logging
RUST_LOG=debug cargo test

# Run integration tests
cargo test --test integration_tests

# Run tests with coverage (requires tarpaulin)
cargo tarpaulin --out Html
```

### Writing Tests

#### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculator_addition() {
        let tool = CalculatorTool;
        let args = serde_json::json!({
            "expression": "2 + 2"
        });

        let result = tokio_test::block_on(tool.execute(args)).unwrap();
        assert_eq!(result.content, "4");
    }
}
```

#### Integration Tests

Add to `tests/` directory:

```rust
// tests/agent_integration.rs
use helios_engine::{Agent, Config};

#[tokio::test]
async fn test_agent_with_tools() {
    let config = Config::from_file("config.test.toml").unwrap();
    let mut agent = Agent::builder("TestAgent")
        .config(config)
        .tool(Box::new(CalculatorTool))
        .build()
        .await
        .unwrap();

    let response = agent.chat("What is 10 + 5?").await.unwrap();
    assert!(response.contains("15"));
}
```

### Test Configuration

Create `config.test.toml` for testing:

```toml
[llm]
model_name = "gpt-3.5-turbo"
base_url = "https://api.openai.com/v1"
api_key = "test-key"
temperature = 0.0  # Deterministic for testing
max_tokens = 100
```

## 📚 Documentation

### Documentation Standards

- Use Markdown for all documentation
- Include code examples where relevant
- Provide both conceptual and practical information
- Keep documentation up-to-date with code changes
- Use clear, concise language accessible to different experience levels

### Updating Documentation

1. **API Documentation**: Update `docs/API.md` for any public API changes
2. **Guides**: Update relevant guides in `docs/` directory
3. **Examples**: Add examples to `examples/` directory
4. **README**: Update main README.md for major changes

### Documentation Checklist

- [ ] Public API changes documented
- [ ] Breaking changes clearly marked
- [ ] Code examples tested and working
- [ ] Cross-references updated
- [ ] Table of contents accurate

## 🛠️ Code Quality

### Formatting

```bash
# Format all code
cargo fmt

# Check formatting without changing files
cargo fmt --check
```

### Linting

```bash
# Run clippy linter
cargo clippy

# Fix auto-fixable issues
cargo clippy --fix
```

### Code Standards

- Follow Rust naming conventions
- Use meaningful variable and function names
- Add documentation comments for public APIs
- Handle errors gracefully
- Write comprehensive tests

### Performance Guidelines

- Minimize allocations in hot paths
- Use async/await appropriately
- Consider memory usage for large data structures
- Profile performance-critical code
- Use appropriate data structures

## 🔧 Tool Development

### Adding New Tools

1. **Implement the Tool trait:**
   ```rust
   use async_trait::async_trait;
   use helios_engine::{Tool, ToolParameter, ToolResult};

   struct MyTool;

   #[async_trait]
   impl Tool for MyTool {
       fn name(&self) -> &str { "my_tool" }

       fn description(&self) -> &str {
           "Description of what my tool does"
       }

       fn parameters(&self) -> HashMap<String, ToolParameter> {
           // Define tool parameters
       }

       async fn execute(&self, args: Value) -> Result<ToolResult> {
           // Implement tool logic
       }
   }
   ```

2. **Add to tool registry** in `tools.rs`

3. **Write comprehensive tests**

4. **Update documentation** in `docs/TOOLS.md`

### Tool Best Practices

- Validate all inputs
- Handle errors gracefully
- Provide meaningful error messages
- Document parameter formats
- Consider security implications
- Test edge cases thoroughly

## 🏛️ Architecture Guidelines

### Module Organization

- `agent.rs`: Agent implementation and builder pattern
- `chat.rs`: Chat messages and session management
- `config.rs`: Configuration loading and validation
- `error.rs`: Error types and handling
- `llm.rs`: LLM client and provider implementations
- `tools.rs`: Tool registry and implementations
- `serve.rs`: HTTP server for API endpoints

### Design Principles

- **Separation of Concerns**: Each module has a single responsibility
- **Dependency Injection**: Use constructor injection for dependencies
- **Error Handling**: Use `Result<T, Error>` consistently
- **Async/Await**: Use async/await for I/O operations
- **Type Safety**: Leverage Rust's type system

### API Design

- Use builder patterns for complex construction
- Provide sensible defaults
- Make APIs hard to misuse
- Document preconditions and postconditions
- Version APIs appropriately

## 🔒 Security

### Security Checklist

- [ ] Input validation on all user inputs
- [ ] No hardcoded secrets or credentials
- [ ] Safe handling of file paths
- [ ] Proper error message sanitization
- [ ] No sensitive data in logs
- [ ] Secure default configurations

### Reporting Security Issues

- **DO NOT** create public GitHub issues for security vulnerabilities
- Email security concerns to: [security email or maintainer contact]
- Provide detailed reproduction steps
- Allow time for fixes before public disclosure

## 🚀 Release Process

### Version Numbering

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Release Checklist

- [ ] Update version in `Cargo.toml`
- [ ] Update changelog
- [ ] Run full test suite
- [ ] Update documentation
- [ ] Create git tag
- [ ] Publish to crates.io
- [ ] Create GitHub release

### Publishing to Crates.io

```bash
# Update version
cargo release [patch|minor|major]

# Or manually:
cargo test
cargo publish
```

## 📞 Communication

### Discussion Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and discussions
- **Pull Request Comments**: Code review discussions

### Getting Help

- Check existing issues and documentation first
- Use clear, descriptive issue titles
- Provide minimal reproduction cases
- Include relevant version information

## 🎯 Areas for Contribution

### High Priority

- [ ] Performance optimizations
- [ ] Additional LLM provider support
- [ ] More built-in tools
- [ ] Improved error messages
- [ ] Better documentation

### Medium Priority

- [ ] Web UI interface
- [ ] Plugin system for tools
- [ ] Advanced RAG features
- [ ] Multi-modal support
- [ ] Integration with popular frameworks

### Future Ideas

- [ ] Mobile SDKs
- [ ] Desktop applications
- [ ] Cloud deployment templates
- [ ] Advanced orchestration features

## 📋 Code of Conduct

### Our Standards

- Be respectful and inclusive
- Focus on constructive feedback
- Help newcomers learn
- Maintain professional communication
- Respect differing viewpoints

### Enforcement

Violations of the code of conduct may result in:
- Warning
- Temporary ban
- Permanent ban from the project

## 🙏 Recognition

Contributors are recognized through:
- GitHub contributor statistics
- Mention in release notes
- Attribution in documentation
- Community recognition

Thank you for contributing to Helios Engine! 🎉
