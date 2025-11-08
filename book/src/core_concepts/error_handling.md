# Error Handling

The Helios Engine uses a custom error type called `HeliosError` to represent all possible errors that can occur within the framework. This chapter will cover the different types of errors and how to handle them.

## `HeliosError`

The `HeliosError` enum is the single error type used throughout the Helios Engine. It has the following variants:

- **`ConfigError(String)`**: An error related to configuration.
- **`LLMError(String)`**: An error related to the Language Model (LLM).
- **`ToolError(String)`**: An error related to a tool.
- **`AgentError(String)`**: An error related to an agent.
- **`NetworkError(#[from] reqwest::Error)`**: An error related to a network request.
- **`SerializationError(#[from] serde_json::Error)`**: An error related to serialization or deserialization.
- **`IoError(#[from] std::io::Error)`**: An I/O error.
- **`TomlError(#[from] toml::de::Error)`**: An error related to parsing TOML.
- **`LlamaCppError(String)`**: An error from the Llama C++ backend (only available with the `local` feature).

## `Result<T>`

The Helios Engine also provides a convenient `Result` type alias that uses the `HeliosError` as the error type:

```rust
pub type Result<T> = std::result::Result<T, HeliosError>;
```

This means that you can use the `?` operator to propagate errors in your code, just like you would with a standard `std::result::Result`.

## Handling Errors

Here's an example of how to handle errors in the Helios Engine:

```rust
use helios_engine::{Agent, Config, CalculatorTool, Result};

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
    }
}

async fn run() -> Result<()> {
    let config = Config::from_file("config.toml")?;

    let mut agent = Agent::builder("MathAgent")
        .config(config)
        .system_prompt("You are a helpful math assistant.")
        .tool(Box::new(CalculatorTool))
        .max_iterations(5)
        .build()
        .await?;

    let response = agent.chat("What is 15 * 8 + 42?").await?;
    println!("Agent: {}", response);

    Ok(())
}
```

In this example, the `run` function returns a `Result<()>`. If any of the operations within the function fail, the error will be propagated up to the `main` function, where it will be printed to the console.
