# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a knowledge management and spaced repetition learning platform built in Rust, implementing the FSRS (Free Spaced Repetition Scheduler) algorithm. The system combines wiki-style knowledge cards with LLM-powered quiz generation and grading.

### Core Architecture
- **Backend**: Rust with SQLite database
- **Spaced Repetition**: FSRS algorithm via rs-fsrs library
- **Knowledge Structure**: Zettelkasten-inspired card linking system
- **Quiz System**: LLM-generated questions with automated grading
- **Frontend**: Modern responsive web UI with comprehensive UX improvements

### UI/UX Features (Phase 4 In Progress)
- **Visual Hierarchy**: Icon-based navigation with Feather Icons, consistent typography scale, and design system
- **Interactive Elements**: Keyboard shortcuts (Space, 1-4 for ratings, ? for help), enhanced loading states, toast notifications
- **Search & Discovery**: Real-time search with highlighting, case-insensitive matching, 300ms debouncing
- **Card Management**: Preview system for long content (100+ chars), expand/collapse functionality
- **Responsive Design**: Mobile-first approach, touch-friendly interfaces, optimized for all screen sizes
- **Progress Tracking**: Visual progress bars, session statistics, completion celebrations
- **Accessibility**: High contrast support, reduced motion preferences, semantic HTML structure
- **Card Headers**: Enhanced Zettel ID badges with gradient styling, improved title typography, compact metadata row
- **Visual Link Indicators**: Wiki-style links with Feather icons, hover effects, and clickable navigation

### Database Design
- **cards**: Core knowledge units with wiki-style links
- **topics**: Hierarchical organization system  
- **card_topics**: Many-to-many relationship mapping
- **reviews**: FSRS statistics and review history tracking
- **backlinks**: Bidirectional linking system for reverse card relationships

**Database Layer Improvements (2025-01-27)**:
- Refactored card mapping to eliminate code duplication
- Added centralized `map_row_to_card()` method in `Database` struct
- Reduced duplicate code by 60+ lines across 4 database methods
- Improved maintainability with single source of truth for card row mapping

**API Layer Improvements (2025-01-27)**:
- Implemented centralized error handling system with `ApiError` enum and `ErrorContext`
- Eliminated inconsistent error handling patterns across API endpoints
- Added structured error classification and consistent HTTP status code mapping
- Standardized logging levels and messages for better debugging
- Improved user experience with friendly error messages while preserving technical details
- Refactored large `start_review_session()` function into 4 focused, testable components
- Applied Single Responsibility Principle to improve code maintainability
- Enhanced function modularity enabling independent unit testing and code reuse

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

### Card Title Migration
```bash
# One-time migration to extract titles from existing markdown headers
cargo run --bin migrate_titles -- --dry-run  # Preview changes
cargo run --bin migrate_titles                # Apply migration
```

## Key Development Considerations

### FSRS Integration
The system uses the rs-fsrs crate for spaced repetition scheduling. Review intervals and difficulty ratings are managed through FSRS state updates with **single update per card** after all questions are completed. Individual question ratings are collected and averaged to determine the final FSRS rating. Keyboard shortcuts (1-4) map directly to FSRS rating levels (Again, Hard, Good, Easy).

### Card Linking System
Cards support wiki-style bidirectional linking with automatic backlink maintenance. When Card A links to Card B, Card B automatically shows Card A in its backlinks section. The linking mechanism preserves referential integrity through foreign key constraints and supports efficient graph traversal for related content discovery. Non-existent link targets are gracefully handled.

### LLM Integration Points
- **OpenAI Responses API**: Updated from chat/completions to modern /responses endpoint with improved request/response structure
- **Batch Quiz Generation**: Multiple cards processed in single API calls for efficiency
- **Session-Based Answer Grading**: Context-aware grading using actual questions from session storage with suggested rating display (deferred FSRS updates)
- **Rating Aggregation**: Individual question ratings are collected and averaged for single FSRS update per card
- **Semantic Answer Grading**: Advanced grading with comprehensive understanding prompts
- **Smart Card Ordering**: Content similarity and difficulty-based optimization for better LLM context
- **Structured Logging**: Comprehensive performance monitoring and debugging capabilities with response ID tracking
- **Fallback Systems**: Triple redundancy (batch → individual → local) ensures reliability

### Database Constraints
- SQLite is used for simplicity and portability
- FSRS statistics are updated once per card after all questions are completed (prevents multiple overwrites)
- Card content supports LaTeX/MathJax markup for mathematical expressions (both inline `$...$` and display `$$...$$`)
- Cards support optional titles extracted from markdown headers or manually entered
- Search functionality uses case-insensitive LIKE queries for content matching
- Backlinks maintain referential integrity through CASCADE DELETE foreign key constraints

### Frontend Architecture
- **Design System**: CSS custom properties for consistent spacing, typography, and colors
- **Component Structure**: Modular card preview system, skeleton loading states, toast notifications
- **State Management**: JavaScript-based with real-time search debouncing and progress tracking
- **Icon System**: Feather Icons with SVG sprites for scalable, accessible iconography
- **Responsive Strategy**: Mobile-first CSS with progressive enhancement for larger screens
- **Review Interface**: Collapsible card display showing titles by default with expandable content sections, batch question display with simultaneous answering
- **Testing Framework**: Custom lightweight JavaScript test framework with 150+ test cases, browser-based test runner, and comprehensive coverage of unit tests, integration tests, and user workflows

## Project Status
**Phase 4 In Progress**: Advanced UI enhancements building on completed efficiency optimizations. Card Header Enhancement completed with modern Zettel ID badges and improved visual hierarchy. Remaining Phase 4 features include linking system visibility, information architecture improvements, and enhanced link experience.

### ✅ Completed Features
- **Navigation**: Icon-enhanced navigation with Feather Icons (layers, refresh-cw, tag icons)
- **Search**: Real-time search with debouncing, highlighting, and case-insensitive matching  
- **Cards**: Preview system for long content with expand/collapse functionality
- **Linking**: Bidirectional backlinks with automatic maintenance and distinct visual styling
- **Math Rendering**: Full LaTeX support with inline `$...$` and display `$$...$$` math
- **Reviews**: Progress tracking, keyboard shortcuts (Space, 1-4), completion celebrations, suggested rating display, aggregated FSRS updates, and batch question display
- **Responsive**: Mobile-first design with touch-friendly interactions
- **Feedback**: Toast notifications, skeleton loading, and enhanced error handling
- **Accessibility**: High contrast mode, reduced motion, semantic HTML
- **Efficiency**: Batch processing for 85-90% reduction in API calls per session
- **Smart Ordering**: Multi-factor card optimization for better LLM context
- **Performance Monitoring**: Comprehensive structured logging and error tracking
- **Centralized Logging**: File-based logging with daily rotation in `logs/learning-system.log.YYYY-MM-DD`
- **Session Answer Submission**: Context-aware grading for multiple choice questions with proper question context and suggested rating display
- **Card Headers (Phase 4)**: Enhanced Zettel ID badges with gradient styling, improved title typography, compact metadata row with icons
- **Visual Link Indicators (Phase 4)**: Wiki-style links enhanced with Feather link icons, modern styling, hover effects, and navigation functionality
- **Batch Question Interface**: All questions for a card displayed simultaneously with batch submission, comprehensive results display, and performance-based suggested ratings

### API Enhancements
- **Search Endpoint**: `/api/cards/search?q={query}` with URL encoding support
- **Linking Endpoints**: `/api/cards/:id/links` and `/api/cards/:id/backlinks` for bidirectional navigation
- **Enhanced Responses**: Consistent JSON structure with success/error states
- **Performance**: Optimized for skeleton loading and progressive enhancement
- **Batch Processing**: `/api/review/session/start` with pre-generated questions for entire sessions
- **Session Answer Endpoint**: `/api/review/session/:session_id/answer/:card_id` for context-aware answer grading
- **Legacy Endpoint Cleanup**: Removed unused individual quiz generation, deprecated problematic answer submission
- **Efficiency Endpoints**: Optimized card ordering and batch question generation

### Testing Coverage
- **Integration Tests**: Search functionality, UI preview, keyboard shortcuts, comprehensive backlink scenarios
- **API Tests**: Search endpoints, response structure validation, URL encoding, backlinks API
- **Database Tests**: Backlink CRUD operations, foreign key constraints, cascade deletions, refactored card mapping functionality
- **Error Handling Tests**: Centralized error classification, context creation, consistent response mapping
- **UI Feature Tests**: Navigation icons, loading states, responsive design validation
- **Error Handling**: Edge cases, concurrent operations, invalid data handling, non-existent link targets
- **Efficiency Tests**: Batch processing, smart ordering algorithms, fallback mechanisms
- **Performance Tests**: API call reduction, content bucketing, overdue ratio calculations
- **Session Answer Tests**: Context-aware answer submission and multiple choice grading validation with suggested rating display
- **Suggested Rating Tests**: UI response format validation for suggested rating display functionality
- **FSRS Integration Tests**: Single update per card validation, rating aggregation, and deferred update functionality
- **Batch Interface Tests**: Simultaneous question display, answer validation, batch submission, and comprehensive result display
- **Frontend Tests (2025-09-08)**: Comprehensive JavaScript test suite with 150+ test cases covering core functionality, search, wiki links, review sessions, and complete user workflows

### Development Notes
- All Phase 2 UI/UX improvements from UI_IMPROVEMENTS.md completed
- All Phase 3 efficiency optimizations from QUIZ_EFFICIENCY_IMPROVEMENTS.md completed
- Phase 4 Card Header Enhancement from UI_IMPROVEMENTS_PHASE_4.md completed
- Phase 4 Visual Link Indicators from UI_IMPROVEMENTS_PHASE_4.md completed
- Bidirectional backlinks system fully implemented and tested
- LaTeX rendering supports both inline and display math expressions
- Batch processing infrastructure with robust fallback mechanisms
- Smart card ordering with multi-factor optimization algorithms
- Comprehensive structured logging throughout the system
- Session-based answer submission fixes multiple choice grading issue with deferred FSRS updates
- Rating aggregation system prevents FSRS overwrite issues and provides balanced difficulty assessment
- Legacy endpoint cleanup removes unused and problematic API routes
- Design system provides consistent foundation for future development
- Mobile-first responsive approach supports modern device usage patterns
- Database layer refactoring eliminates 60+ lines of duplicate card mapping code (2025-01-27)
- API error handling centralization with structured error types and consistent logging (2025-01-27)
- Large function refactoring improves testability with 54% complexity reduction (2025-01-27)
- OpenAI Responses API migration from /chat/completions to /responses with improved structure (2025-01-27)
- **Backend Tests**: 118 total Rust tests covering all functionality including FSRS integration, rating aggregation, batch question interface, database refactoring, centralized error handling, and modular function architecture
- **Frontend Tests**: 150+ JavaScript test cases with custom test framework covering unit tests, integration tests, user workflows, performance, and accessibility

### Performance Achievements
- **API Call Reduction**: 85-90% fewer calls per review session (from ~15-20 to 1-2 calls)
- **Session Initialization**: Instant question access through batch pre-generation
- **Smart Ordering**: Optimized card sequence for better LLM context utilization
- **Context-Aware Grading**: Multiple choice questions graded against actual question context with AI-powered suggested ratings
- **FSRS Optimization**: Single database update per card with aggregated ratings (prevents overwrite issues)
- **Monitoring**: Comprehensive structured logging for performance visibility

**Latest Update (2025-01-27)**: OpenAI Responses API Migration - Updated from /chat/completions to modern /responses endpoint with enhanced request structure using `input` field instead of messages array. Improved response parsing with status validation and response ID logging for better debugging. All 143 tests pass with full backward compatibility maintained.

**Previous Update**: Batch Question Interface completed - all questions for a card now display simultaneously with batch submission, comprehensive scoring, and performance-based suggested ratings. This significantly improves review efficiency and user experience.
**Current Phase**: Phase 4 UI enhancements in progress (link preview cards, collapsible sections, visual hierarchy improvements)
- The newest models for GPT are gpt-5 and gpt-5-mini