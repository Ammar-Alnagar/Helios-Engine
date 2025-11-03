# 🚀 Helios Engine Features

## 🎯 Overview

Helios Engine is a powerful and flexible Rust framework for building LLM-powered agents with tool support, streaming chat capabilities, and easy configuration management. Create intelligent agents that can interact with users, call tools, and maintain conversation context - with both online and offline local model support.

## 🔧 Core Features

### Agent System
- **Multi-Agent Architecture**: Create multiple agents with different personalities, capabilities, and configurations
- **Agent Builder Pattern**: Easy-to-use builder for creating agents with custom system prompts and tools
- **Persistent Conversation Context**: Maintains conversation history and context for meaningful interactions
- **Tool Integration**: Agents can be equipped with various tools to extend their capabilities

### Tool Support
Helios Engine comes with 18+ built-in tools for various tasks:

#### File Operations
- **File Read/Write Tools**: Read and write files with support for line ranges and directory creation
- **File Search Tool**: Search for files by name patterns or content with recursive directory traversal
- **File Edit Tool**: Edit files by replacing specific text or content
- **File List Tool**: List directory contents with detailed information
- **File I/O Tool**: Unified interface for common file operations (read, write, append, delete, copy, move)

#### System & Information Tools
- **System Info Tool**: Retrieve OS, CPU, memory, disk, and network information
- **Timestamp Tool**: Work with timestamps, date/time operations, formatting, and time arithmetic
- **JSON Parser Tool**: Parse, validate, format, and manipulate JSON data with path operations
- **Text Processor Tool**: Advanced text operations like search, replace, split, join, count, case conversion

#### Network & Web Tools
- **Web Scraper Tool**: Fetch and extract content from web URLs with HTML text extraction
- **HTTP Request Tool**: Make HTTP requests with various methods (GET, POST, PUT, DELETE, PATCH)
- **Shell Command Tool**: Execute shell commands with safety restrictions

#### Data & Memory Tools
- **In-Memory Database Tool**: Key-value store for agents to cache data during conversations
- **Calculator Tool**: Perform basic arithmetic operations (+, -, *, /)
- **Echo Tool**: Echo back provided messages

#### Advanced Tools
- **Qdrant RAG Tool**: RAG (Retrieval-Augmented Generation) system with vector database support
- **RAG Tool**: Integration with the RAG system for knowledge retrieval
- **Multi-Agent Communication Tools**: Tools for agents to send messages, delegate tasks, and share context

### Multi-Agent Collaboration (Forest of Agents)
- **Agent Communication**: Agents can send messages to each other and broadcast to all agents
- **Task Delegation**: Agents can delegate tasks to other specialized agents
- **Shared Context**: All agents have access to shared context and can update it
- **Collaborative Task Execution**: Complex tasks can be distributed across multiple agents
- **Message History**: Track all communication between agents for context and debugging

### RAG (Retrieval-Augmented Generation) System
- **Vector Stores**: Support for both in-memory and Qdrant vector stores
- **Embedding Providers**: Integration with OpenAI embeddings and support for local models
- **Document Management**: Add, search, delete, and manage documents with metadata
- **Semantic Search**: Perform similarity searches to find relevant information
- **Document Chunking**: Automatic document chunking and preprocessing for better retrieval

### Streaming Support
- **Real-time Response Streaming**: True real-time response streaming for both remote and local models
- **Immediate Token Delivery**: Immediate delivery of tokens as they're generated
- **Dual Mode Support**: Auto, online (remote API), and offline (local) modes

### Local Model Support
- **llama.cpp Integration**: Run local models offline using llama.cpp with HuggingFace integration
- **Feature Flag Support**: Optional `local` feature for offline model support
- **No Remote Dependency**: Full functionality available without external API calls

### HTTP Server & API
- **OpenAI-Compatible API**: Expose OpenAI-compatible API endpoints
- **Full Parameter Support**: Support for all standard OpenAI parameters
- **Custom Endpoints**: Support for custom API endpoints and configurations
- **CORS Support**: Built-in CORS and trace support

### Configuration Management
- **Flexible Configuration**: Easy configuration management with TOML files
- **Multiple Provider Support**: Works with OpenAI, Azure OpenAI, and local models
- **Environment Variables**: Support for API keys through environment variables
- **Custom Endpoints**: Support for custom endpoint configurations

## 🏗️ Architecture

### Modular Design
- **Modular Architecture**: Clean separation of concerns with dedicated modules for agents, tools, RAG, and more
- **Extensible Tool System**: Easy to create and register custom tools
- **Trait-Based Architecture**: Trait-based design for extensibility and testing

### Error Handling
- **Custom Error Types**: Comprehensive error handling with custom `HeliosError` type
- **Result Type**: Consistent use of `Result` type throughout the codebase
- **Detailed Error Messages**: Clear and informative error messages

### Async Support
- **Tokio Integration**: Built on Tokio for high-performance async operations
- **Async Traits**: Use of async traits for tools and other components
- **Concurrent Operations**: Support for concurrent operations and requests

## 🛠️ Development Features

### CLI Interface
- **Interactive Chat**: Start interactive chat sessions with agents
- **Quick Question Feature**: Ask quick questions without starting a full session
- **Configuration Initialization**: Initialize configuration files with required API keys
- **Command-line Options**: Full range of command-line options for different modes

### Library Integration
- **Crate Usage**: Can be used as a Rust library in other projects
- **Builder Pattern**: Consistent use of builder pattern for object creation
- **Easy Integration**: Simple integration with existing Rust applications

### Testing
- **Comprehensive Test Coverage**: Extensive unit and integration tests
- **Test-Driven Development**: Test-driven approach with detailed test cases
- **Mock Support**: Support for testing with mocked components

## 🚀 Usage Modes

### Online Mode
- **Remote API Support**: Connect to OpenAI, Azure OpenAI, and other compatible APIs
- **Real-time Processing**: Real-time processing with streaming responses
- **Cloud Integration**: Full integration with cloud-based LLM services

### Offline Mode
- **Local Model Support**: Run models locally without internet connection
- **Privacy First**: Keep sensitive data on local machines
- **Cost Effective**: No API costs when running local models

### Auto Mode
- **Automatic Mode Detection**: Automatically choose between online and offline based on configuration
- **Fallback Support**: Automatic fallback if primary mode fails

## 🔐 Security Features

### Safe Operations
- **Safe Shell Commands**: Restricted shell commands with dangerous operation blocking
- **File Operation Safety**: Safe file operations with permission checks
- **Input Validation**: Comprehensive input validation and sanitization

### Security Best Practices
- **API Key Security**: Secure handling of API keys and sensitive information
- **Parameter Sanitization**: Input sanitization for all user-provided parameters
- **Rate Limiting**: Built-in rate limiting for API calls