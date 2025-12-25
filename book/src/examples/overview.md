# Examples Overview

This directory contains comprehensive examples demonstrating various features of the Helios Engine framework.

## Table of Contents

- [Running Examples](#running-examples)
- [Basic Examples](#basic-examples)
- [Agent Examples](#agent-examples)
- [Advanced Examples](#advanced-examples)
- [RAG Examples](#rag-examples)
- [API Examples](#api-examples)

## Running Examples

All examples can be run using Cargo:

```bash
# Run a specific example
cargo run --example basic_chat

# List all available examples
cargo run --example --list
```

### Individual Example Commands

```bash
# Basic chat example
cargo run --example basic_chat

# Agent with built-in tools (Calculator, Echo)
cargo run --example agent_with_tools

# Agent with file management tools
cargo run --example agent_with_file_tools

# Agent with in-memory database tool
cargo run --example agent_with_memory_db

# Custom tool implementation
cargo run --example custom_tool

# Multiple agents with different personalities
cargo run --example multiple_agents

# Forest of Agents - collaborative multi-agent system
cargo run --example forest_of_agents

# Forest with Coordinator - enhanced planning system
cargo run --example forest_with_coordinator

# Forest Simple Demo - simple reliable demo of planning system
cargo run --example forest_simple_demo

# Direct LLM usage without agents
cargo run --example direct_llm_usage

# Streaming chat with remote models
cargo run --example streaming_chat

# Local model streaming example
cargo run --example local_streaming

# Serve an agent via HTTP API
cargo run --example serve_agent

# Serve with custom endpoints
cargo run --example serve_with_custom_endpoints

# SendMessageTool demo - test messaging functionality
cargo run --example send_message_tool_demo

# Agent with RAG capabilities
cargo run --example agent_with_rag

# RAG with in-memory vector store
cargo run --example rag_in_memory

# Compare RAG implementations (Qdrant vs InMemory)
cargo run --example rag_qdrant_comparison

# Complete demo with all features
cargo run --example complete_demo
```
