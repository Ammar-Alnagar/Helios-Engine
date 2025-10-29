# Publishing Guide for Helios

This guide will help you publish Helios to crates.io so it can be used both as a library and as a CLI tool.

## Pre-Publishing Checklist

### 1. Update Cargo.toml Metadata

Make sure to update these fields in `Cargo.toml`:

```toml
[package]
authors = ["Your Name <your.email@example.com>"]  # Update with your info
repository = "https://github.com/yourusername/helios"  # Update with your repo
homepage = "https://github.com/yourusername/helios"  # Update with your repo
```

### 2. Ensure All Files Are Ready

- [ ] `README.md` is up to date
- [ ] `LICENSE` file exists (MIT license is already included)
- [ ] `CHANGELOG.md` is updated with latest changes
- [ ] Examples are working and documented
- [ ] All documentation is accurate

### 3. Test the Package Locally

```bash
# Check that everything compiles
cargo check
cargo test

# Test the library
cargo build --lib

# Test the binary
cargo build --bin helios

# Run examples
cargo run --example basic_chat
cargo run --example direct_llm_usage

# Check for warnings
cargo clippy -- -W clippy::all

# Check documentation
cargo doc --open
```

### 4. Verify Package Contents

```bash
# Create a package without uploading
cargo package --list

# This shows all files that will be included
# Make sure no unwanted files are included
```

## Publishing Steps

### 1. Login to crates.io

First, you need a crates.io account. Get your API token from https://crates.io/me

```bash
cargo login <your-api-token>
```

### 2. Dry Run

Test the publishing process without actually uploading:

```bash
cargo publish --dry-run
```

This will:
- Build your package
- Check for errors
- Show what would be uploaded
- NOT actually publish

### 3. Publish to crates.io

Once the dry run succeeds:

```bash
cargo publish
```

🎉 Your crate is now published!

### 4. Verify the Publication

- Check your crate at `https://crates.io/crates/helios`
- Documentation will be automatically built at `https://docs.rs/helios`
- Wait a few minutes for docs to build

## After Publishing

### Installing as a CLI Tool

Users can now install Helios as a command-line tool:

```bash
cargo install helios
```

Then use it:

```bash
helios
```

### Using as a Library

Users can add it to their `Cargo.toml`:

```toml
[dependencies]
helios = "0.1.0"
tokio = { version = "1.35", features = ["full"] }
```

And use it in code:

```rust
use helios::{LLMClient, ChatMessage};
use helios::config::LLMConfig;

#[tokio::main]
async fn main() -> helios::Result<()> {
    let llm_config = LLMConfig {
        model_name: "gpt-3.5-turbo".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        api_key: std::env::var("OPENAI_API_KEY").unwrap(),
        temperature: 0.7,
        max_tokens: 2048,
    };

    let client = LLMClient::new(llm_config);
    let messages = vec![
        ChatMessage::user("Hello!"),
    ];

    let response = client.chat(messages, None).await?;
    println!("{}", response.content);
    Ok(())
}
```

## Versioning

Helios follows [Semantic Versioning](https://semver.org/):

- **MAJOR** version (1.x.x) - incompatible API changes
- **MINOR** version (x.1.x) - add functionality in a backwards compatible manner
- **PATCH** version (x.x.1) - backwards compatible bug fixes

### Publishing a New Version

1. Update version in `Cargo.toml`:
   ```toml
   version = "0.2.0"  # or whatever the new version is
   ```

2. Update `CHANGELOG.md` with changes

3. Commit the changes:
   ```bash
   git add Cargo.toml CHANGELOG.md
   git commit -m "Bump version to 0.2.0"
   git tag v0.2.0
   git push && git push --tags
   ```

4. Publish:
   ```bash
   cargo publish
   ```

## Yanking a Version (If Needed)

If you published a broken version:

```bash
cargo yank --vers 0.1.0
```

To un-yank:

```bash
cargo yank --vers 0.1.0 --undo
```

## Including/Excluding Files

### Files Automatically Included

- `Cargo.toml`
- `src/**`
- `README.md`
- `LICENSE` or `LICENSE-*`
- `examples/**`

### Files Automatically Excluded

- `.git/`
- `target/`
- Hidden files (`.gitignore`, etc.)

### Custom Exclusions

Add to `Cargo.toml`:

```toml
[package]
exclude = [
    "tests/fixtures/*",
    "tmp_*",
]
```

Or specify what to include:

```toml
[package]
include = [
    "src/**/*",
    "examples/**/*",
    "docs/**/*",
    "Cargo.toml",
    "README.md",
    "LICENSE",
]
```

## Troubleshooting

### Error: "crate name is already taken"

If the name `helios` is taken, you'll need to choose a different name:

1. Update `name` in `Cargo.toml`
2. Update `lib.name` and `bin.name`
3. Update all documentation
4. Publish with the new name

### Error: "failed to verify package tarball"

This usually means there are compilation errors or missing files:

```bash
cargo clean
cargo build
cargo publish --dry-run
```

### Error: "some files in the working directory contain changes"

Commit or stash your changes:

```bash
git add -A
git commit -m "Prepare for publishing"
```

Or use:

```bash
cargo publish --allow-dirty
```

(Not recommended for actual publishing)

## Best Practices

1. **Test Before Publishing**: Always run `cargo publish --dry-run` first
2. **Version Carefully**: Once published, a version cannot be changed (only yanked)
3. **Document Everything**: Good documentation increases adoption
4. **Semantic Versioning**: Follow semver strictly
5. **Keep CHANGELOG**: Maintain a detailed changelog
6. **Test Examples**: Ensure all examples work
7. **CI/CD**: Set up GitHub Actions to test before publishing

## Example GitHub Actions Workflow

Create `.github/workflows/publish.yml`:

```yaml
name: Publish to crates.io

on:
  push:
    tags:
      - 'v*'

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      - uses: actions-rs/cargo@v1
        with:
          command: publish
          args: --token ${{ secrets.CARGO_TOKEN }}
```

Add your crates.io token as a GitHub secret named `CARGO_TOKEN`.

## Resources

- [Cargo Book - Publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [crates.io](https://crates.io/)
- [docs.rs](https://docs.rs/)
- [Semantic Versioning](https://semver.org/)

## Support

If you have questions about publishing:
- Check the [Cargo Book](https://doc.rust-lang.org/cargo/)
- Ask in [Rust Users Forum](https://users.rust-lang.org/)
- Join [Rust Discord](https://discord.gg/rust-lang)
