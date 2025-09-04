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

# Ensure the directory for the database exists and is writable
DB_DIR=$(dirname "${DATABASE_URL#sqlite:}")
if [ ! -d "$DB_DIR" ]; then
  echo "Creating database directory: $DB_DIR"
  mkdir -p "$DB_DIR"
fi

# Check if we can write to the database directory
if [ ! -w "$DB_DIR" ]; then
  echo "Warning: Database directory $DB_DIR is not writable"
  echo "Attempting to fix permissions..."
  chmod 755 "$DB_DIR" 2>/dev/null || echo "Could not fix permissions - you may need to run with appropriate permissions"
fi

# You can set your LLM configuration here or via environment
# 
# OpenAI Configuration:
# export LLM_PROVIDER="openai"
# export LLM_API_KEY="sk-your-openai-key-here"
# export LLM_MODEL="gpt-4o-mini"
# export LLM_BASE_URL="https://api.openai.com/v1"
#
# Google Gemini Configuration:
# export LLM_PROVIDER="gemini"
# export LLM_API_KEY="AIza-your-gemini-key-here"
# export LLM_MODEL="gemini-2.5-flash"
# export LLM_BASE_URL="https://generativelanguage.googleapis.com/v1beta"
#
# Local LLM (Ollama) Configuration:
# export LLM_PROVIDER="openai"
# export LLM_API_KEY="ollama"
# export LLM_BASE_URL="http://localhost:11434/v1"

# Logging configuration examples:
# export RUST_LOG="debug"                          # Debug everything
# export RUST_LOG="info"                           # Info level for all
# export RUST_LOG="error"                          # Error level only
# export RUST_LOG="info,learning_system=debug"    # Default: Info for deps, Debug for app

# Create logs directory if it doesn't exist
if [ ! -d "logs" ]; then
  echo "Creating logs directory..."
  mkdir -p logs
fi

# Check if Cargo.toml exists (verify we're in the right directory)
if [ ! -f "Cargo.toml" ]; then
  echo "Error: Cargo.toml not found. Please run this script from the project root directory."
  exit 1
fi

# Validate database path
DB_PATH="${DATABASE_URL#sqlite:}"
if [ "$DB_PATH" != "$DATABASE_URL" ]; then
  # It's a SQLite database
  DB_DIR=$(dirname "$DB_PATH")
  if [ ! -w "$DB_DIR" ]; then
    echo "Error: Cannot write to database directory: $DB_DIR"
    echo "Please check permissions or run with appropriate privileges."
    exit 1
  fi
fi

echo "Starting Learning System..."
echo "Database: $DATABASE_URL"
echo "Port: $PORT"
echo "LLM Provider: ${LLM_PROVIDER:-openai}"
echo "LLM Model: ${LLM_MODEL:-default}"
echo "LLM Base URL: ${LLM_BASE_URL:-default}"
echo "Log Level: $RUST_LOG"
echo "Logs written to: logs/learning-system.log.$(date +%Y-%m-%d)"
echo ""

# Build and run the application
echo "Building and starting application..."
cargo run --release

