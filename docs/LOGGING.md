# Logging System Documentation

## Overview

The Learning System implements comprehensive centralized logging using the `tracing` ecosystem with both console and file output capabilities.

## Features

- **Dual Output**: Simultaneous logging to console and file
- **Daily Log Rotation**: Automatic daily rotation with timestamped files  
- **Structured Logging**: Rich contextual information with thread IDs, file locations, and line numbers
- **Environment Configuration**: Flexible log level control via `RUST_LOG` environment variable
- **Performance Monitoring**: Detailed logging for batch operations, API calls, and efficiency metrics

## Log File Location

All logs are written to: `logs/learning-system.log.YYYY-MM-DD`

Examples:
- `logs/learning-system.log.2025-08-31`
- `logs/learning-system.log.2025-09-01`

## Log Levels

The system supports standard tracing log levels:

1. **ERROR**: Critical errors and failures
2. **WARN**: Warnings and fallback operations
3. **INFO**: General application events and status updates
4. **DEBUG**: Detailed debugging information including database queries
5. **TRACE**: Maximum verbosity for deep debugging

## Configuration

### Environment Variable

Use the `RUST_LOG` environment variable to control logging:

```bash
# Basic levels
RUST_LOG=info                           # INFO level for all modules
RUST_LOG=debug                          # DEBUG level for all modules  
RUST_LOG=error                          # ERROR level only

# Module-specific logging
RUST_LOG=info,learning_system=debug     # INFO for dependencies, DEBUG for our app
RUST_LOG=learning_system::llm_service=trace  # TRACE level for specific module

# Default if not set
RUST_LOG=info,learning_system=debug
```

### Production Recommendations

- **Development**: `RUST_LOG=debug` or `RUST_LOG=info,learning_system=debug`
- **Staging**: `RUST_LOG=info`
- **Production**: `RUST_LOG=warn` or `RUST_LOG=error`

## Log Format

Each log entry includes:
- **Timestamp**: ISO 8601 format with microsecond precision
- **Level**: Log level (INFO, DEBUG, WARN, ERROR)
- **Thread ID**: For concurrency debugging
- **Module**: Target module path
- **Location**: File and line number
- **Message**: Structured log message with contextual fields

Example:
```
2025-08-31T17:50:07.988821Z  INFO ThreadId(01) learning_system: src/main.rs:46: Starting Learning System server...
```

## Key Logging Areas

### 1. Batch Question Generation
```rust
info!(
    card_count = cards.len(),
    card_ids = ?cards.iter().map(|c| c.id).collect::<Vec<_>>(),
    "Generating batch quiz questions for multiple cards"
);
```

### 2. Smart Card Ordering
```rust
info!(
    card_count = cards.len(),
    "Applying smart ordering to due cards for optimal batch processing"
);
```

### 3. LLM API Interactions
```rust
debug!(
    card_count = cards.len(),
    response_content = %content,
    "Raw LLM response for batch quiz generation"
);
```

### 4. Error Handling
```rust
error!(
    card_id = %card.id,
    status = %status,
    error = %error_text,
    "LLM API request failed for quiz generation"
);
```

## Log Analysis

### Monitoring Performance
Look for these key metrics in logs:
- API call reduction: Search for "batch" operations vs individual calls
- Session initialization times: Track "Starting" to "Database initialized" 
- LLM response times: Monitor API request/response cycles
- Fallback usage: Watch for "falling back" messages

### Debugging Issues
- **Quiz Generation**: Filter by `card_id` or "quiz generation"
- **Batch Processing**: Search for "batch" or "fallback"
- **Database Operations**: Look for `sqlx::query` entries (DEBUG level)
- **API Errors**: Filter by ERROR level and specific endpoints

## Log Rotation

- **Automatic**: Daily rotation at midnight UTC
- **File naming**: `learning-system.log.YYYY-MM-DD`
- **Retention**: Old logs are kept indefinitely (manual cleanup recommended)

## Disk Space Management

Log files are excluded from version control via `.gitignore`. Consider implementing log retention policies:

```bash
# Keep logs for 30 days (example cleanup script)
find logs/ -name "*.log.*" -mtime +30 -delete
```

## Integration with Monitoring

The structured logging format is compatible with:
- **ELK Stack**: Elasticsearch, Logstash, Kibana
- **Grafana**: Log aggregation and visualization
- **Datadog**: APM and log monitoring
- **CloudWatch**: AWS log aggregation

## Troubleshooting

### Common Issues

1. **No log files created**: Check write permissions on `logs/` directory
2. **Missing logs**: Verify `RUST_LOG` environment variable is set
3. **Too much/little detail**: Adjust log levels per module
4. **Performance impact**: Lower log levels in production

### Debug Commands

```bash
# Check current log configuration
echo $RUST_LOG

# View recent logs
tail -f logs/learning-system.log.$(date +%Y-%m-%d)

# Monitor real-time logs with filtering  
tail -f logs/learning-system.log.$(date +%Y-%m-%d) | grep "batch"

# Count error entries
grep "ERROR" logs/learning-system.log.$(date +%Y-%m-%d) | wc -l
```

## Security Considerations

- **Sensitive Data**: Logging avoids including API keys, user passwords, or PII
- **Content Truncation**: Long card content is truncated in logs to prevent sensitive data exposure
- **File Permissions**: Log files inherit directory permissions (ensure appropriate access)

The logging system provides comprehensive visibility into application behavior while maintaining security and performance best practices.