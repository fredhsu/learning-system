# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a knowledge management and spaced repetition learning platform built in Rust, implementing the FSRS (Free Spaced Repetition Scheduler) algorithm. The system combines wiki-style knowledge cards with LLM-powered quiz generation and grading.

### Core Architecture
- **Backend**: Rust with SQLite database
- **Spaced Repetition**: FSRS algorithm via rs-fsrs library
- **Knowledge Structure**: Zettelkasten-inspired card linking system
- **Quiz System**: LLM-generated questions with automated grading
- **Frontend**: Web-based SPA/SSR with MathJax support

### Database Design
- **cards**: Core knowledge units with wiki-style links
- **topics**: Hierarchical organization system  
- **card_topics**: Many-to-many relationship mapping
- **reviews**: FSRS statistics and review history tracking

## Development Commands

### Build and Run
```bash
cargo build          # Build the project
cargo run            # Run the application
cargo check          # Fast syntax/type checking
```

### Testing
```bash
cargo test           # Run all tests
cargo test --lib     # Run library tests only
cargo test --bin     # Run binary tests only
```

### Code Quality
```bash
cargo clippy         # Linting
cargo fmt            # Code formatting
```

## Key Development Considerations

### FSRS Integration
The system uses the rs-fsrs crate for spaced repetition scheduling. Review intervals and difficulty ratings are managed through FSRS state updates after each quiz attempt.

### Card Linking System
Cards support wiki-style bidirectional linking. The linking mechanism should preserve referential integrity and support efficient graph traversal for related content discovery.

### LLM Integration Points
- Quiz question generation from card content
- Answer grading with structured JSON responses
- Prompt templates require careful design for consistent outputs

### Database Constraints
- SQLite is used for simplicity and portability
- FSRS statistics must be atomically updated with review submissions
- Card content supports LaTeX/MathJax markup for mathematical expressions

## Project Status
Currently in initial setup phase with basic Rust project structure. The main.rs contains only a "Hello, world!" placeholder.