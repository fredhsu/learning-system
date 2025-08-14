#!/bin/bash
# Learning System Launch Script

# Load environment variables from .env file if it exists
if [ -f .env ]; then
    echo "Loading environment from .env file..."
    export $(cat .env | grep -v '^#' | xargs)
fi

# Set default environment variables if not already set
export DATABASE_URL="${DATABASE_URL:-sqlite:./learning_system.db}"
export PORT="${PORT:-3000}"

# You can set your LLM API key here or via environment
# export LLM_API_KEY="your-openai-api-key-here"
# export LLM_BASE_URL="https://api.openai.com/v1"

# For local LLM (Ollama), use:
# export LLM_API_KEY="ollama"
# export LLM_BASE_URL="http://localhost:11434/v1"

echo "Starting Learning System..."
echo "Database: $DATABASE_URL"
echo "Port: $PORT"
echo "LLM Base URL: ${LLM_BASE_URL:-https://api.openai.com/v1}"

cargo run --release