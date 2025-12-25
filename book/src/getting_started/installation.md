# Installation

You can install Helios Engine as a command-line tool or use it as a library in your own Rust projects.

## As a CLI Tool

### Standard Installation
To install the CLI tool without local model support (which is lighter and faster to install), run the following command:

```bash
cargo install helios-engine
```

### With Local Model Support
If you want to use Helios Engine with local models, you'll need to install it with the `local` feature enabled. This will also install `llama-cpp-2` and its dependencies.

```bash
cargo install helios-engine --features local
```

## As a Library

To use Helios Engine as a library in your own project, add the following to your `Cargo.toml` file:

```toml
[dependencies]
helios-engine = "0.4.3"
tokio = { version = "1.35", features = ["full"] }
```
