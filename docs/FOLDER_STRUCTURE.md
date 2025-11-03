# Helios Folder Structure

Complete directory and file structure of the Helios project.

## 📁 Directory Tree

```
helios/
│
├── 📄 Cargo.toml                    # Project manifest and dependencies
├── 📄 Cargo.lock                    # Dependency lock file (auto-generated)
├── 📄 .gitignore                    # Git ignore patterns
│
├── 📄 LICENSE                       # MIT License
├── 📄 README.md                     # Main documentation (start here!)
├── 📄 PROJECT_OVERVIEW.md           # High-level project summary
├── 📄 ARCHITECTURE.md               # Technical architecture details
├── 📄 CONTRIBUTING.md               # Contribution guidelines
├── 📄 CHANGELOG.md                  # Version history and changes
├── 📄 FOLDER_STRUCTURE.md           # This file
│
├── 📄 config.example.toml           # Example configuration file
│
├── 📂 src/                          # Source code (library + binary)
│   ├── 📄 lib.rs                   # Library entry point (public API)
│   ├── 📄 main.rs                  # Binary entry point (CLI demo)
│   ├── 📄 agent.rs                 # Agent system implementation
│   ├── 📄 llm.rs                   # LLM client and provider
│   ├── 📄 tools.rs                 # Tool system and built-in tools
│   ├── 📄 chat.rs                  # Chat messages and sessions
│   ├── 📄 config.rs                # Configuration management
│   └── 📄 error.rs                 # Error types and handling
│
├── 📂 examples/                     # Example programs
│   ├── 📄 basic_chat.rs            # Simple chat example
│   ├── 📄 agent_with_tools.rs      # Tool usage demonstration
│   ├── 📄 custom_tool.rs           # Custom tool implementation
│   └── 📄 multiple_agents.rs       # Multiple agent coordination
│
├── 📂 docs/                         # Additional documentation
│   ├── 📄 QUICKSTART.md            # 5-minute quick start guide
│   ├── 📄 TUTORIAL.md              # Comprehensive tutorial
│   └── 📄 API.md                   # Complete API reference
│
└── 📂 target/                       # Build artifacts (ignored by git)
    ├── debug/                       # Debug build output
    └── release/                     # Release build output
```

## 📋 File Descriptions

### Root Level Files

| File | Lines | Purpose |
|------|-------|---------|
| `Cargo.toml` | ~40 | Project configuration, dependencies, build settings |
| `Cargo.lock` | Auto | Locked dependency versions (auto-generated) |
| `.gitignore` | ~10 | Files and directories to ignore in version control |
| `LICENSE` | ~21 | MIT License text |
| `README.md` | ~600 | Main documentation with examples and diagrams |
| `PROJECT_OVERVIEW.md` | ~400 | High-level project summary and statistics |
| `ARCHITECTURE.md` | ~300 | Detailed technical architecture |
| `CONTRIBUTING.md` | ~200 | Guidelines for contributors |
| `CHANGELOG.md` | ~100 | Version history and release notes |
| `FOLDER_STRUCTURE.md` | ~250 | This file - directory structure |
| `config.example.toml` | ~10 | Example configuration file |

### Source Files (`src/`)

| File | Lines | Purpose | Exports |
|------|-------|---------|---------|
| `lib.rs` | ~15 | Library entry point | All public APIs |
| `main.rs` | ~80 | Interactive CLI demo | Binary executable |
| `agent.rs` | ~200 | Agent system | `Agent`, `AgentBuilder` |
| `llm.rs` | ~150 | LLM client | `LLMClient`, `LLMProvider` |
| `tools.rs` | ~250 | Tool system | `Tool`, `ToolRegistry`, built-in tools |
| `chat.rs` | ~130 | Chat management | `ChatMessage`, `ChatSession`, `Role` |
| `config.rs` | ~80 | Configuration | `Config`, `LLMConfig` |
| `error.rs` | ~30 | Error types | `HeliosError`, `Result<T>` |

**Total Source Lines**: ~935 lines of Rust code

### Example Files (`examples/`)

| File | Lines | Purpose |
|------|-------|---------|
| `basic_chat.rs` | ~20 | Simple chat interaction |
| `agent_with_tools.rs` | ~30 | Demonstrates tool usage |
| `custom_tool.rs` | ~60 | Shows how to create custom tools |
| `multiple_agents.rs` | ~40 | Multiple specialized agents |

**Total Example Lines**: ~150 lines

### Documentation Files (`docs/`)

| File | Lines | Purpose |
|------|-------|---------|
| `QUICKSTART.md` | ~150 | Fast-track getting started guide |
| `TUTORIAL.md` | ~600 | In-depth tutorial with examples |
| `API.md` | ~400 | Complete API reference |

**Total Documentation Lines**: ~1,150 lines

## 🗂️ Module Organization

### Public API Structure

```
helios
│
├── agent
│   ├── Agent
│   └── AgentBuilder
│
├── config
│   ├── Config
│   └── LLMConfig
│
├── llm
│   ├── LLMClient
│   └── LLMProvider (trait)
│
├── tools
│   ├── Tool (trait)
│   ├── ToolRegistry
│   ├── ToolParameter
│   ├── ToolResult
│   ├── CalculatorTool
│   └── EchoTool
│
├── chat
│   ├── ChatMessage
│   ├── ChatSession
│   └── Role
│
└── error
    ├── HeliosError
    └── Result<T>
```

## 📊 Project Statistics

### Code Distribution

```
Source Code (src/)         :  935 lines (42%)
Examples (examples/)       :  150 lines (7%)
Documentation (*.md)       : 2,300 lines (51%)
────────────────────────────────────────────
Total                      : 3,385 lines
```

### File Count by Type

```
Rust Source Files (.rs)    :  12 files
Markdown Docs (.md)        :  10 files
Config Files (.toml)       :   2 files
Other (.gitignore, etc)    :   1 file
────────────────────────────────────────────
Total                      :  25 files
```

### Directory Count

```
Root directories           :   3 (src, examples, docs)
Total directories          :   3 (excluding target)
```

## 🔍 File Dependencies

### Module Dependency Graph

```
main.rs
  └── lib.rs
       ├── agent.rs
       │    ├── config.rs
       │    ├── llm.rs
       │    ├── tools.rs
       │    ├── chat.rs
       │    └── error.rs
       │
       ├── llm.rs
       │    ├── config.rs
       │    ├── chat.rs
       │    ├── tools.rs
       │    └── error.rs
       │
       ├── tools.rs
       │    └── error.rs
       │
       ├── chat.rs
       │
       ├── config.rs
       │    └── error.rs
       │
       └── error.rs
```

## 📦 Build Artifacts

### Debug Build (`target/debug/`)

```
target/debug/
├── helios                  # Debug binary (CLI demo)
├── libhelios.rlib         # Debug library
├── deps/                  # Dependency artifacts
└── examples/              # Compiled examples
    ├── basic_chat
    ├── agent_with_tools
    ├── custom_tool
    └── multiple_agents
```

### Release Build (`target/release/`)

```
target/release/
├── helios                  # Optimized binary
├── libhelios.rlib         # Optimized library
├── deps/                  # Dependency artifacts
└── examples/              # Compiled examples
    ├── basic_chat
    ├── agent_with_tools
    ├── custom_tool
    └── multiple_agents
```

## 🎯 Entry Points

### Library Usage

```rust
// In your Cargo.toml
[dependencies]
helios-engine = "0.3.7"

// In your code
use helios_engine::{Agent, Config};
```

**Entry Point**: `src/lib.rs`

### Binary Usage

```bash
# Build and run the interactive demo
cargo run

# Or run directly
./target/release/helios
```

**Entry Point**: `src/main.rs`

### Examples

```bash
# Run any example
cargo run --example basic_chat
cargo run --example agent_with_tools
cargo run --example custom_tool
cargo run --example multiple_agents
```

**Entry Points**: Files in `examples/` directory

## 🔧 Configuration Files

### User Configuration

```
config.toml                 # User's actual config (not in git)
config.example.toml         # Template configuration
```

Create your config:
```bash
cp config.example.toml config.toml
# Edit config.toml with your API key
```

## 📝 Documentation Hierarchy

### Reading Order for New Users

1. **README.md** - Overview and features
2. **docs/QUICKSTART.md** - Get started in 5 minutes
3. **docs/TUTORIAL.md** - Learn by doing
4. **docs/API.md** - Reference documentation

### Reading Order for Contributors

1. **CONTRIBUTING.md** - How to contribute
2. **ARCHITECTURE.md** - Technical design
3. **PROJECT_OVERVIEW.md** - Project summary
4. **CHANGELOG.md** - Version history

## 🚀 Build Commands

### Development

```bash
cargo build                 # Debug build
cargo run                   # Run CLI demo
cargo test                  # Run tests
cargo check                 # Quick compile check
cargo clippy                # Linting
cargo fmt                   # Format code
```

### Production

```bash
cargo build --release       # Optimized build
cargo test --release        # Release tests
cargo doc                   # Generate documentation
cargo doc --open            # Generate and open docs
```

### Examples

```bash
cargo build --examples      # Build all examples
cargo run --example NAME    # Run specific example
```

## 📐 Size Metrics

### Source Code

```
agent.rs     : ~200 lines (Largest module)
tools.rs     : ~250 lines (Most complex)
llm.rs       : ~150 lines
chat.rs      : ~130 lines
config.rs    : ~80 lines
error.rs     : ~30 lines (Smallest module)
```

### Documentation

```
README.md    : ~600 lines (Most comprehensive)
TUTORIAL.md  : ~600 lines (Most detailed)
API.md       : ~400 lines
ARCHITECTURE.md : ~300 lines
```

## 🎨 Visual Representation

```
           helios/
              │
    ┌─────────┼─────────┐
    │         │         │
   src/    examples/  docs/
    │         │         │
    ├─8 files ├─4 files ├─3 files
    │         │         │
   Core    Demos    Guides
```

## ✅ Completeness Checklist

- [x] Source code (8 files)
- [x] Examples (4 files)
- [x] Documentation (10 files)
- [x] Configuration (2 files)
- [x] License (MIT)
- [x] Build system (Cargo)
- [x] Version control (.gitignore)

## 📚 Navigation Guide

### Want to...

- **Use Helios?** Start with `README.md`
- **Quick start?** Read `docs/QUICKSTART.md`
- **Learn deeply?** Follow `docs/TUTORIAL.md`
- **Understand design?** See `ARCHITECTURE.md`
- **Contribute?** Check `CONTRIBUTING.md`
- **See examples?** Browse `examples/` directory
- **API reference?** Read `docs/API.md`

---

**Last Updated**: 2024
**Project Status**: ✅ Complete and Ready

This structure provides a clean, organized, and well-documented Rust project! 🚀
