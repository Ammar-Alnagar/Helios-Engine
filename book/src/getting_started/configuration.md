# Configuration

Before you can start using Helios Engine, you'll need to create a `config.toml` file to store your API keys and other settings.

## Initializing Configuration

The easiest way to get started is to use the `init` command:

```bash
helios-engine init
```

This will create a `config.toml` file in your current directory with the following content:

```toml
[llm]
model_name = "gpt-3.5-turbo"
base_url = "https://api.openai.com/v1"
api_key = "your-api-key-here"
temperature = 0.7
max_tokens = 2048
```

You'll need to replace `"your-api-key-here"` with your actual API key.

## Common Providers

Here are some examples of how to configure Helios Engine for different LLM providers:

### OpenAI
```toml
[llm]
base_url = "https://api.openai.com/v1"
model_name = "gpt-4"
api_key = "sk-..."
```

### Local (LM Studio)
```toml
[llm]
base_url = "http://localhost:1234/v1"
model_name = "local-model"
api_key = "not-needed"
```

### Ollama
```toml
[llm]
base_url = "http://localhost:11434/v1"
model_name = "llama2"
api_key = "not-needed"
```

### Anthropic
```toml
[llm]
base_url = "https://api.anthropic.com/v1"
model_name = "claude-3-opus-20240229"
api_key = "sk-ant-..."
```
