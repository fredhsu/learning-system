#!/bin/bash
# Learning System Launch Script

# Load environment variables from .env file if it exists
if [ -f .env ]; then
  echo "Loading environment from .env file..."
  export $(cat .env | grep -v '^#' | xargs)
fi

# Set default environment variables if not already set
export DATABASE_URL="${DATABASE_URL:-sqlite:./learning_system.db}"
export PORT="${PORT:-4000}"
export RUST_LOG="${RUST_LOG:-info,learning_system=debug}"

# You can set your LLM API key here or via environment
# export LLM_API_KEY="your-openai-api-key-here"
# export LLM_BASE_URL="https://api.openai.com/v1"

# For local LLM (Ollama), use:
# export LLM_API_KEY="ollama"
# export LLM_BASE_URL="http://localhost:11434/v1"

# Logging configuration examples:
# export RUST_LOG="debug"                          # Debug everything
# export RUST_LOG="info"                           # Info level for all
# export RUST_LOG="error"                          # Error level only
# export RUST_LOG="info,learning_system=debug"    # Default: Info for deps, Debug for app

echo "Starting Learning System..."
echo "Database: $DATABASE_URL"
echo "Port: $PORT"
echo "LLM Base URL: ${LLM_BASE_URL:-https://api.openai.com/v1}"
echo "Log Level: $RUST_LOG"
echo "Logs written to: logs/learning-system.log.$(date +%Y-%m-%d)"

cargo run --release

