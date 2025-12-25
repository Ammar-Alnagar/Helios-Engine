# Using Tools

Tools allow agents to perform actions beyond just text generation, enabling them to interact with files, execute commands, access web resources, and manipulate data. This chapter will cover the built-in tools and how to use them.

## Built-in Tools

Helios Engine includes 16+ built-in tools for common tasks. Here's an overview of the most common ones:

### Core Tools

#### CalculatorTool
Performs mathematical calculations and evaluations.

```rust
use helios_engine::CalculatorTool;

let mut agent = Agent::builder("MathAgent")
    .config(config)
    .tool(Box::new(CalculatorTool))
    .build()
    .await?;
```

**Parameters:**
- `expression` (string, required): Mathematical expression to evaluate

**Example Usage:**
```rust
let result = agent.chat("Calculate 15 * 7 + 3").await?;
```

#### EchoTool
Simply echoes back the input message (useful for testing).

```rust
use helios_engine::EchoTool;

agent.tool(Box::new(EchoTool));
```

**Parameters:**
- `message` (string, required): Message to echo back

### File Management Tools

#### FileSearchTool
Search for files by name pattern or content within files.

```rust
use helios_engine::FileSearchTool;

agent.tool(Box::new(FileSearchTool));
```

**Parameters:**
- `path` (string, optional): Directory path to search (default: current directory)
- `pattern` (string, optional): File name pattern with wildcards (e.g., `*.rs`)
- `content` (string, optional): Text content to search for within files
- `max_results` (number, optional): Maximum number of results (default: 50)

#### FileReadTool
Read the contents of a file with optional line range selection.

```rust
use helios_engine::FileReadTool;

agent.tool(Box::new(FileReadTool));
```

**Parameters:**
- `path` (string, required): File path to read
- `start_line` (number, optional): Starting line number (1-indexed)
- `end_line` (number, optional): Ending line number (1-indexed)

#### FileWriteTool
Write content to a file (creates new or overwrites existing).

```rust
use helios_engine::FileWriteTool;

agent.tool(Box::new(FileWriteTool));
```

**Parameters:**
- `path` (string, required): File path to write to
- `content` (string, required): Content to write

#### FileEditTool
Edit a file by replacing specific text (find and replace).

```rust
use helios_engine::FileEditTool;

agent.tool(Box::new(FileEditTool));
```

**Parameters:**
- `path` (string, required): File path to edit
- `find` (string, required): Text to find
- `replace` (string, required): Replacement text

### Web & API Tools

#### WebScraperTool
Fetch and extract content from web URLs.

```rust
use helios_engine::WebScraperTool;

agent.tool(Box::new(WebScraperTool));
```

**Parameters:**
- `url` (string, required): URL to scrape
- `extract_text` (boolean, optional): Extract readable text from HTML
- `timeout_seconds` (number, optional): Request timeout

#### HttpRequestTool
Make HTTP requests with various methods.

```rust
use helios_engine::HttpRequestTool;

agent.tool(Box::new(HttpRequestTool));
```

**Parameters:**
- `method` (string, required): HTTP method (GET, POST, PUT, DELETE, etc.)
- `url` (string, required): Request URL
- `headers` (object, optional): Request headers
- `body` (string, optional): Request body
- `timeout_seconds` (number, optional): Request timeout

### System & Utility Tools

#### ShellCommandTool
Execute shell commands safely with security restrictions.

```rust
use helios_engine::ShellCommandTool;

agent.tool(Box::new(ShellCommandTool));
```

**Parameters:**
- `command` (string, required): Shell command to execute
- `timeout_seconds` (number, optional): Command timeout

#### SystemInfoTool
Retrieve system information (OS, CPU, memory, disk, network).

```rust
use helios_engine::SystemInfoTool;

agent.tool(Box::new(SystemInfoTool));
```

**Parameters:**
- `category` (string, optional): Info category (all, os, cpu, memory, disk, network)
