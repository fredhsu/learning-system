# CLAUDE.md

This file provides guidance to Claude Code when working with this repository.

## Project Overview

A knowledge management and spaced repetition learning platform built in Rust with FSRS algorithm integration. Features wiki-style cards with LLM-powered quiz generation and parallel grading.

### Core Architecture
- **Backend**: Rust with SQLite database
- **Spaced Repetition**: FSRS algorithm via rs-fsrs library
- **Knowledge Structure**: Zettelkasten-inspired card linking system
- **Quiz System**: LLM-generated questions with automated grading
- **Frontend**: Modern responsive web UI with comprehensive UX improvements

### Features
- **Navigation**: Icon-based UI with Feather Icons, keyboard shortcuts (Space, 1-4), progress tracking
- **Search**: Real-time search with highlighting and debouncing
- **Cards**: Preview/expand system, wiki-style bidirectional linking, LaTeX math support
- **Reviews**: Batch question display, suggested ratings, FSRS integration
- **Processing**: Parallel/batch/sequential modes with intelligent auto-selection
- **Design**: Mobile-first responsive design, accessibility features, toast notifications

### Database Schema
- **cards**: Knowledge units with wiki-style links
- **topics**: Hierarchical organization
- **reviews**: FSRS statistics and history
- **backlinks**: Bidirectional linking

## Development Commands

### Build and Run
```bash
cargo build          # Build the project
cargo run            # Run the application
cargo check          # Fast syntax/type checking
```

### Logging
```bash
# Run with different log levels
RUST_LOG=debug cargo run                     # Debug level logging
RUST_LOG=info,learning_system=debug cargo run  # Targeted logging
RUST_LOG=error cargo run                     # Error level only

# View logs
tail -f logs/learning-system.log.$(date +%Y-%m-%d)  # Follow current logs
grep "batch" logs/learning-system.log.*              # Search across all logs
```

### Testing
```bash
# Backend tests (Rust)
cargo test           # Run all tests
cargo test --lib     # Run library tests only
cargo test --bin     # Run binary tests only

# Frontend tests (JavaScript)
# Open static/test-runner.html in browser for interactive testing
# Tests run automatically on page load with detailed reporting
```

### Code Quality
```bash
cargo clippy         # Linting
cargo fmt            # Code formatting
```


## Key Development Considerations

### Core Systems
- **FSRS**: Spaced repetition with single update per card, averaged ratings from multiple questions
- **Linking**: Bidirectional wiki-style links with automatic backlink maintenance
- **Processing**: Parallel/batch/sequential modes with intelligent selection and fallback chains

### LLM Integration
- **OpenAI API**: Chat completions with structured prompts and response parsing
- **Processing Modes**: Parallel (concurrent tokio::spawn), batch (single API call), sequential (individual calls)
- **Intelligent Selection**: Auto mode based on question count (1→sequential, 2→batch, 3+→parallel)
- **Fallback Chain**: Parallel → batch → sequential → local for 100% reliability
- **Performance**: 40-75% improvement for multi-question cards, configurable concurrency (1-10 tasks)

### Technical Details
- **Database**: SQLite with LaTeX support, cascade constraints, case-insensitive search
- **Frontend**: JavaScript with Feather Icons, mobile-first responsive design, custom test framework (150+ tests)
- **Backend**: Rust with comprehensive error handling, structured logging, 98 tests (97 passing)

## Project Status
**Phase 2 Complete**: Parallel processing implemented with 40-75% performance improvement. Phase 4 UI enhancements in progress.

### ✅ Completed Features
- **UI**: Icon navigation, real-time search, card preview/expand, mobile-first responsive design
- **Reviews**: Batch questions, keyboard shortcuts, progress tracking, suggested ratings
- **Linking**: Bidirectional wiki-style links with visual indicators
- **Processing**: Parallel/batch/sequential modes with auto-selection and fallback
- **Performance**: 40-75% faster grading, 85-90% fewer API calls
- **Math**: LaTeX rendering support
- **Testing**: 150+ frontend tests, 98 backend tests (97 passing)

### Key API Endpoints
- `/api/cards/search?q={query}` - Real-time search
- `/api/cards/:id/links` and `/api/cards/:id/backlinks` - Bidirectional linking
- `/api/review/session/start` - Batch question generation
- `/api/review/session/:session_id/answers/:card_id/parallel` - Parallel grading
- `/api/review/session/:session_id/answer/:card_id` - Context-aware grading

### Testing
- **Backend**: 98 Rust tests covering API, database, FSRS, parallel processing, error handling
- **Frontend**: 150+ JavaScript tests covering UI, search, reviews, parallel processing
- **Coverage**: Search, linking, batch processing, parallel grading, fallback mechanisms


### Performance
- **85-90% fewer API calls** per session via batch processing
- **40-75% faster grading** for multi-question cards via parallel processing
- **Intelligent mode selection** optimizes processing based on question count
- **Configurable concurrency** (1-10 tasks) prevents system overload

**Latest Update (2025-09-09)**: Phase 2 parallel processing complete with true concurrent grading, intelligent auto-selection, comprehensive fallback chain, and extensive test coverage.