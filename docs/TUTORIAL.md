# Helios Tutorial

A comprehensive tutorial for building LLM agents with Helios.

## Table of Contents

1. [Introduction](#introduction)
2. [Basic Concepts](#basic-concepts)
3. [Building Your First Agent](#building-your-first-agent)
4. [Working with Tools](#working-with-tools)
5. [Creating Custom Tools](#creating-custom-tools)
6. [Advanced Features](#advanced-features)
7. [Best Practices](#best-practices)

## Introduction

Helios is a Rust framework for building LLM-powered agents with tool support. This tutorial will guide you through creating progressively more sophisticated agents.

## Basic Concepts

### What is an Agent?

An agent is an autonomous entity that:
- Receives user input
- Processes it using an LLM
- Can call tools to perform actions
- Returns responses to the user

### Key Components

```
┌─────────────────────────────────────────┐
│              Agent                      │
│  ┌────────────────────────────────┐    │
│  │     LLM Client                 │    │
│  │  (Communicates with AI)        │    │
│  └────────────────────────────────┘    │
│  ┌────────────────────────────────┐    │
│  │     Tool Registry              │    │
│  │  (Available tools)             │    │
│  └────────────────────────────────┘    │
│  ┌────────────────────────────────┐    │
│  │     Chat Session               │    │
│  │  (Conversation history)        │    │
│  └────────────────────────────────┘    │
└─────────────────────────────────────────┘
```

## Building Your First Agent

### Step 1: Setup

Create a new Rust project:
```bash
cargo new my-helios-agent
cd my-helios-agent
```

Add dependencies to `Cargo.toml`:
```toml
[dependencies]
helios-engine = "0.3.7"
tokio = { version = "1.35", features = ["full"] }
```

### Step 2: Configuration

Create `config.toml`:
```toml
[llm]
model_name = "gpt-3.5-turbo"
base_url = "https://api.openai.com/v1"
api_key = "your-api-key-here"
temperature = 0.7
max_tokens = 2048
```

### Step 3: Simple Chat Agent

```rust
use helios_engine::{Agent, Config};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    // Load configuration
    let config = Config::from_file("config.toml")?;

    // Create agent
    let mut agent = Agent::builder("ChatBot")
        .config(config)
        .system_prompt("You are a friendly chatbot.")
        .build()
        .await?;

    // Single interaction
    let response = agent.chat("Tell me a joke").await?;
    println!("{}", response);

    Ok(())
}
```

Run it:
```bash
cargo run
```

### Step 4: Conversational Agent

Add conversation memory:

```rust
use helios_engine::{Agent, Config};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;

    let mut agent = Agent::builder("ChatBot")
        .config(config)
        .system_prompt("You are a friendly chatbot with good memory.")
        .build()
        .await?;

    println!("Chat with the bot (type 'exit' to quit):");

    loop {
        print!("\nYou: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input == "exit" {
            break;
        }

        let response = agent.chat(input).await?;
        println!("Bot: {}", response);
    }

    Ok(())
}
```

Try it:
```
You: My name is Alice
Bot: Nice to meet you, Alice!

You: What's my name?
Bot: Your name is Alice!
```

## Working with Tools

### Built-in Tools

Helios comes with example tools:

```rust
use helios_engine::{Agent, Config, CalculatorTool, EchoTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;

    let mut agent = Agent::builder("ToolBot")
        .config(config)
        .system_prompt(
            "You are a helpful assistant with tools. \
             Use the calculator for math and echo for repeating messages."
        )
        .tool(Box::new(CalculatorTool))
        .tool(Box::new(EchoTool))
        .build()
        .await?;

    // Agent will automatically use calculator
    let response = agent.chat("What is 15 * 8 + 12?").await?;
    println!("{}", response);

    // Agent will use echo
    let response = agent.chat("Please echo: Hello World").await?;
    println!("{}", response);

    Ok(())
}
```

### How Tool Calling Works

1. User sends a message
2. Agent sends message + tool definitions to LLM
3. LLM decides if it needs a tool
4. If yes, returns tool call request
5. Agent executes the tool
6. Agent sends tool result back to LLM
7. LLM generates final response
8. Agent returns to user

## Creating Custom Tools

### Simple Tool Example

```rust
use async_trait::async_trait;
use helios_engine::{Tool, ToolParameter, ToolResult};
use serde_json::Value;
use std::collections::HashMap;

struct GreetingTool;

#[async_trait]
impl Tool for GreetingTool {
    fn name(&self) -> &str {
        "greet"
    }

    fn description(&self) -> &str {
        "Generate a personalized greeting"
    }

    fn parameters(&self) -> HashMap<String, ToolParameter> {
        let mut params = HashMap::new();
        params.insert(
            "name".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "The person's name".to_string(),
                required: Some(true),
            },
        );
        params.insert(
            "language".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Language: 'english', 'spanish', 'french'".to_string(),
                required: Some(false),
            },
        );
        params
    }

    async fn execute(&self, args: Value) -> helios_engine::Result<ToolResult> {
        let name = args["name"].as_str().unwrap_or("Friend");
        let language = args.get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("english");

        let greeting = match language {
            "spanish" => format!("¡Hola, {}!", name),
            "french" => format!("Bonjour, {}!", name),
            _ => format!("Hello, {}!", name),
        };

        Ok(ToolResult::success(greeting))
    }
}
```

### Using Your Custom Tool

```rust
use helios_engine::{Agent, Config};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;

    let mut agent = Agent::builder("GreetBot")
        .config(config)
        .system_prompt("You greet people using the greet tool.")
        .tool(Box::new(GreetingTool))
        .build()
        .await?;

    let response = agent.chat("Greet Alice in Spanish").await?;
    println!("{}", response);

    Ok(())
}
```

### Advanced Tool: API Integration

```rust
use async_trait::async_trait;
use helios_engine::{Tool, ToolParameter, ToolResult};
use serde_json::Value;
use std::collections::HashMap;

struct WeatherTool;

#[async_trait]
impl Tool for WeatherTool {
    fn name(&self) -> &str {
        "get_weather"
    }

    fn description(&self) -> &str {
        "Get current weather for a city"
    }

    fn parameters(&self) -> HashMap<String, ToolParameter> {
        let mut params = HashMap::new();
        params.insert(
            "city".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "City name".to_string(),
                required: Some(true),
            },
        );
        params
    }

    async fn execute(&self, args: Value) -> helios_engine::Result<ToolResult> {
        let city = args["city"].as_str().unwrap_or("Unknown");

        // In a real implementation, you would call a weather API here.
        // For demo purposes, return mock data
        let weather = format!("Weather in {}: Sunny, 72°F", city);

        Ok(ToolResult::success(weather))
    }
}
```

## Advanced Features

### Multiple Specialized Agents

```rust
use helios_engine::{Agent, Config};

struct AgentTeam {
    researcher: Agent,
    writer: Agent,
}

impl AgentTeam {
    async fn new(config: Config) -> helios_engine::Result<Self> {
        let researcher = Agent::builder("Researcher")
            .config(config.clone())
            .system_prompt("You are a research specialist. Provide factual information.")
            .build()
            .await?;

        let writer = Agent::builder("Writer")
            .config(config)
            .system_prompt("You are a creative writer. Transform information into engaging content.")
            .build()
            .await?;

        Ok(Self { researcher, writer })
    }

    async fn process(&mut self, topic: &str) -> helios_engine::Result<String> {
        // Research phase
        let research = self.researcher
            .chat(&format!("Research key facts about: {}", topic))
            .await?;

        // Writing phase
        let article = self.writer
            .chat(&format!("Write an article based on: {}", research))
            .await?;

        Ok(article)
    }
}

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;
    let mut team = AgentTeam::new(config).await?;

    let result = team.process("Rust programming language").await?;
    println!("{}", result);

    Ok(())
}
```

### Dynamic Tool Selection

```rust
use helios_engine::{Agent, Config, CalculatorTool, EchoTool};

fn create_agent_with_tools(
    config: Config,
    enable_calculator: bool,
    enable_echo: bool,
) -> helios_engine::Result<Agent> {
    let mut builder = Agent::builder("DynamicAgent")
        .config(config)
        .system_prompt("Use available tools when needed.");

    if enable_calculator {
        builder = builder.tool(Box::new(CalculatorTool));
    }

    if enable_echo {
        builder = builder.tool(Box::new(EchoTool));
    }

    builder.build()
}
```

### Conversation Management

```rust
use helios_engine::{Agent, Config};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;
    let mut agent = Agent::builder("Assistant")
        .config(config)
        .build()
        .await?;

    // Long conversation
    for i in 1..100 {
        agent.chat(&format!("Message {}", i)).await?;

        // Clear history every 10 messages to manage memory
        if i % 10 == 0 {
            agent.clear_history();
            println!("History cleared");
        }
    }

    Ok(())
}
```

## Best Practices

### 1. Error Handling

Always handle errors gracefully:

```rust
match agent.chat("Hello").await {
    Ok(response) => {
        println!("Success: {}", response);
    }
    Err(e) => {
        eprintln!("Error occurred: {}", e);
        // Handle error appropriately
    }
}
```

### 2. Configuration Management

Don't hardcode API keys:

```rust
// ❌ Bad
let api_key = "sk-1234...";

// ✅ Good
let config = Config::from_file("config.toml")?;

// ✅ Even better
use std::env;
let api_key = env::var("OPENAI_API_KEY")?;
```

### 3. Tool Design

Make tools focused and reusable:

```rust
// ❌ Bad: Too broad
struct DoEverythingTool;

// ✅ Good: Focused
struct CalculatorTool;
struct WeatherTool;
struct DatabaseTool;
```

### 4. System Prompts

Be specific and clear:

```rust
// ❌ Vague
.system_prompt("Help users")

// ✅ Clear
.system_prompt(
    "You are a helpful customer service agent. \
     Be polite, provide accurate information, \
     and use tools when needed to help users."
)
```

### 5. Iteration Limits

Set appropriate limits:

```rust
let mut agent = Agent::builder("Agent")
    .config(config)
    .max_iterations(5)  // Prevent infinite loops
    .build()
    .await?;
```

### 6. Testing

Test your tools independently:

```rust
#[tokio::test]
async fn test_calculator() {
    let tool = CalculatorTool;
    let args = serde_json::json!({"expression": "2 + 2"});
    let result = tool.execute(args).await.unwrap();
    assert_eq!(result.output, "4");
}
```

## Common Patterns

### Pattern 1: Command Handler

```rust
async fn handle_command(agent: &mut Agent, command: &str) -> helios::Result<()> {
    match command {
        cmd if cmd.starts_with("/clear") => {
            agent.clear_history();
            println!("History cleared");
        }
        cmd if cmd.starts_with("/help") => {
            println!("Available commands: /clear, /help, /exit");
        }
        cmd => {
            let response = agent.chat(cmd).await?;
            println!("Agent: {}", response);
        }
    }
    Ok(())
}
```

### Pattern 2: Retry Logic

```rust
async fn chat_with_retry(
    agent: &mut Agent,
    message: &str,
    max_retries: u32,
) -> helios::Result<String> {
    for attempt in 1..=max_retries {
        match agent.chat(message).await {
            Ok(response) => return Ok(response),
            Err(e) if attempt < max_retries => {
                eprintln!("Attempt {} failed: {}", attempt, e);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}
```

### Pattern 3: Logging

```rust
use tracing::{info, error};

#[tokio::main]
async fn main() -> helios::Result<()> {
    tracing_subscriber::fmt::init();
    
    let config = Config::from_file("config.toml")?;
    let mut agent = Agent::builder("Agent")
        .config(config)
        .build()
    .await?;
    
    info!("Agent initialized");
    
    match agent.chat("Hello").await {
        Ok(response) => {
            info!("Chat successful");
            println!("{}", response);
        }
        Err(e) => {
            error!("Chat failed: {}", e);
        }
    }
    
    Ok(())
}
```

## Next Steps

1. Build a complete application with Helios
2. Create your own custom tools
3. Integrate with databases and APIs
4. Deploy your agent in production
5. Contribute to the Helios project!

## Resources

- [Examples](../examples/)
- [API Documentation](../README.md)
- [Architecture Guide](../ARCHITECTURE.md)
- [Contributing](../CONTRIBUTING.md)

Happy building! 🚀
