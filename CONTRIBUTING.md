# Contributing to Helios

Thank you for your interest in contributing to Helios! This document provides guidelines and instructions for contributing.

## Code of Conduct

- Be respectful and inclusive
- Welcome newcomers and help them learn
- Focus on constructive feedback
- Maintain a positive community

## How to Contribute


### Reporting Bugs

1. Check if the bug has already been reported in [Issues](https://github.com/yourusername/helios/issues)
2. If not, create a new issue with:
   - Clear title and description
   - Steps to reproduce
   - Expected vs actual behavior
   - Environment details (OS, Rust version)
   - Code samples if applicable

### Suggesting Features

1. Check existing feature requests
2. Create a new issue with:
   - Clear use case description
   - Proposed API or behavior
   - Alternative approaches considered
   - Potential impact on existing code

### Submitting Changes

1. **Fork the repository**
   ```bash
   git clone https://github.com/yourusername/helios.git
   cd helios
   ```

2. **Create a branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make your changes**
   - Write clear, documented code
   - Follow Rust conventions
   - Add tests for new functionality
   - Update documentation

4. **Test your changes**
   ```bash
   cargo test
   cargo clippy
   cargo fmt
   ```

5. **Commit your changes**
   ```bash
   git add .
   git commit -m "Add: your feature description"
   ```

6. **Push and create PR**
   ```bash
   git push origin feature/your-feature-name
   ```

## Development Guidelines

### Code Style

- Follow Rust standard naming conventions
- Use `cargo fmt` for formatting
- Address `cargo clippy` warnings
- Write idiomatic Rust code

### Documentation

- Document all public APIs with rustdoc comments
- Include examples in documentation
- Update README.md for major changes
- Add inline comments for complex logic

### Testing

- Write unit tests for new functionality
- Add integration tests for features
- Ensure all tests pass before submitting
- Test edge cases and error conditions

### Commit Messages

Use clear, descriptive commit messages:

```
Add: New feature description
Fix: Bug description
Update: Modification description
Docs: Documentation update
Refactor: Code improvement
Test: Test additions/modifications
```

## Project Structure

```
helios/
├── src/           # Source code
├── examples/      # Example programs
├── tests/         # Integration tests (if added)
└── docs/          # Additional documentation
```

## Building and Testing

### Build
```bash
cargo build
cargo build --release
```

### Test
```bash
cargo test
cargo test --all-features
```

### Check
```bash
cargo check
cargo clippy -- -D warnings
cargo fmt --check
```

### Run Examples
```bash
cargo run --example basic_chat
cargo run --example agent_with_tools
```

## Areas for Contribution

### High Priority

- [ ] Streaming response support
- [ ] More built-in tools
- [ ] Better error messages
- [ ] Performance optimizations
- [ ] More comprehensive tests

### Medium Priority

- [ ] Tool result caching
- [ ] Conversation persistence
- [ ] Metrics and observability
- [ ] Plugin system
- [ ] More examples

### Documentation

- [ ] Video tutorials
- [ ] More code examples
- [ ] API documentation improvements
- [ ] Architecture diagrams
- [ ] Best practices guide

## Creating Custom Tools

When contributing tools, ensure they:

1. Implement the `Tool` trait correctly
2. Have clear descriptions
3. Validate input parameters
4. Handle errors gracefully
5. Include documentation and examples

Example:
```rust
use async_trait::async_trait;
use helios::{Tool, ToolParameter, ToolResult};

/// A tool that converts temperatures
struct TemperatureTool;

#[async_trait]
impl Tool for TemperatureTool {
    fn name(&self) -> &str {
        "convert_temperature"
    }

    fn description(&self) -> &str {
        "Convert temperature between Celsius and Fahrenheit"
    }

    fn parameters(&self) -> HashMap<String, ToolParameter> {
        // Define parameters
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        // Implement conversion logic
    }
}
```

## Getting Help

- 💬 Ask questions in [Discussions](https://github.com/yourusername/helios/discussions)
- 📧 Email: support@helios.dev
- 🐛 Report issues in [Issues](https://github.com/yourusername/helios/issues)

## Recognition

Contributors will be recognized in:
- README.md contributors section
- Release notes
- Project documentation

Thank you for contributing to Helios! 🚀
