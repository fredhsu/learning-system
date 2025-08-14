# Learning System - FSRS Quiz Platform

A knowledge management and spaced repetition learning platform built in Rust, implementing the FSRS (Free Spaced Repetition Scheduler) algorithm with LLM-powered quiz generation.

## Features

- **Spaced Repetition**: Uses the FSRS algorithm for optimal review scheduling
- **Knowledge Cards**: Wiki-style cards with bidirectional linking
- **LLM Integration**: Automated quiz question generation and grading
- **Topics**: Hierarchical organization of knowledge cards
- **LaTeX Support**: Mathematical notation support via MathJax
- **Web Interface**: Modern, responsive web UI
- **RESTful API**: Full REST API for programmatic access

## Architecture

### Backend (Rust)
- **Database**: SQLite with SQLx for data persistence
- **FSRS**: Uses `rs-fsrs` library for spaced repetition scheduling
- **Web Framework**: Axum for REST API and static file serving
- **LLM Integration**: Configurable LLM providers (OpenAI, local models)

### Frontend (Web)
- **SPA**: Single-page application with vanilla JavaScript
- **MathJax**: LaTeX rendering for mathematical content
- **Responsive**: Mobile-friendly design

### Database Schema
- `cards`: Core knowledge units with FSRS statistics
- `topics`: Hierarchical topic organization
- `card_topics`: Many-to-many card-topic relationships
- `reviews`: Historical review data and statistics

## Quick Start

### Prerequisites
- Rust 1.70+
- SQLite 3
- (Optional) OpenAI API key for LLM features

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd learning-system
```

2. Set up environment variables:
```bash
cp .env.example .env
# Edit .env with your configuration
```

3. Build and run:
```bash
cargo build --release
cargo run
```

4. Open your browser to `http://localhost:3000`

### Development

Run in development mode:
```bash
cargo run
```

Run tests:
```bash
cargo test
```

Code formatting:
```bash
cargo fmt
```

Linting:
```bash
cargo clippy
```

## Usage

### Creating Cards
1. Navigate to the "Cards" tab
2. Click "Create New Card"
3. Enter your content (supports LaTeX notation)
4. Assign topics and link to other cards
5. Save the card

### Review Session
1. Navigate to the "Review" tab
2. The system shows cards due for review based on FSRS scheduling
3. Answer the generated quiz questions
4. Rate your performance (Again, Hard, Good, Easy)
5. FSRS automatically schedules the next review

### Topics Management
1. Navigate to the "Topics" tab
2. Create topics to organize your cards
3. Assign cards to multiple topics

## Configuration

### Environment Variables

- `DATABASE_URL`: SQLite database path (default: `sqlite:learning.db`)
- `LLM_API_KEY`: API key for LLM provider
- `LLM_BASE_URL`: LLM provider endpoint (default: OpenAI)
- `PORT`: Server port (default: 3000)

### LLM Providers

#### OpenAI
```env
LLM_API_KEY=sk-your-openai-key
LLM_BASE_URL=https://api.openai.com/v1
```

#### Local LLM (Ollama)
```env
LLM_API_KEY=ollama
LLM_BASE_URL=http://localhost:11434/v1
```

## API Reference

### Cards
- `POST /api/cards` - Create a new card
- `GET /api/cards/:id` - Get a specific card
- `PUT /api/cards/:id` - Update a card
- `GET /api/cards/due` - Get cards due for review
- `GET /api/cards/:id/links` - Get linked cards

### Topics
- `POST /api/topics` - Create a new topic
- `GET /api/topics` - Get all topics

### Quiz & Review
- `GET /api/cards/:id/quiz` - Generate quiz questions for a card
- `POST /api/cards/:id/quiz/answer` - Submit quiz answer
- `POST /api/cards/:id/review` - Record review rating

## FSRS Integration

The system uses the FSRS algorithm for spaced repetition scheduling:

- **Ratings**: 1=Again, 2=Hard, 3=Good, 4=Easy
- **Scheduling**: Cards are automatically scheduled based on performance
- **Statistics**: Difficulty, stability, and retrievability are tracked
- **States**: New → Learning → Review (with possible Relearning)

## Testing

Run the test suite:
```bash
cargo test
```

Tests include:
- Database operations
- FSRS scheduling logic
- Card management workflows
- API endpoint validation

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- [FSRS Algorithm](https://github.com/open-spaced-repetition/fsrs4anki) - The science behind spaced repetition
- [rs-fsrs](https://github.com/open-spaced-repetition/rs-fsrs) - Rust implementation of FSRS
- [MathJax](https://www.mathjax.org/) - LaTeX rendering in the browser