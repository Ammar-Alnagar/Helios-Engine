# How to Contribute

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

## 🧪 Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name
```

## 📚 Documentation

### Documentation Standards

- Use Markdown for all documentation
- Include code examples where relevant
- Provide both conceptual and practical information
- Keep documentation up-to-date with code changes
- Use clear, concise language accessible to different experience levels
