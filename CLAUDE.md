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

### UI/UX Features (Phase 2 Complete)
- **Visual Hierarchy**: Icon-based navigation with Feather Icons, consistent typography scale, and design system
- **Interactive Elements**: Keyboard shortcuts (Space, 1-4 for ratings, ? for help), enhanced loading states, toast notifications
- **Search & Discovery**: Real-time search with highlighting, case-insensitive matching, 300ms debouncing
- **Card Management**: Preview system for long content (100+ chars), expand/collapse functionality
- **Responsive Design**: Mobile-first approach, touch-friendly interfaces, optimized for all screen sizes
- **Progress Tracking**: Visual progress bars, session statistics, completion celebrations
- **Accessibility**: High contrast support, reduced motion preferences, semantic HTML structure

### Database Design
- **cards**: Core knowledge units with wiki-style links
- **topics**: Hierarchical organization system  
- **card_topics**: Many-to-many relationship mapping
- **reviews**: FSRS statistics and review history tracking
- **backlinks**: Bidirectional linking system for reverse card relationships

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
The system uses the rs-fsrs crate for spaced repetition scheduling. Review intervals and difficulty ratings are managed through FSRS state updates after each quiz attempt. Keyboard shortcuts (1-4) map directly to FSRS rating levels (Again, Hard, Good, Easy).

### Card Linking System
Cards support wiki-style bidirectional linking with automatic backlink maintenance. When Card A links to Card B, Card B automatically shows Card A in its backlinks section. The linking mechanism preserves referential integrity through foreign key constraints and supports efficient graph traversal for related content discovery. Non-existent link targets are gracefully handled.

### LLM Integration Points
- **Batch Quiz Generation**: Multiple cards processed in single API calls for efficiency
- **Session-Based Answer Grading**: Context-aware grading using actual questions from session storage
- **Semantic Answer Grading**: Advanced grading with comprehensive understanding prompts
- **Smart Card Ordering**: Content similarity and difficulty-based optimization for better LLM context
- **Structured Logging**: Comprehensive performance monitoring and debugging capabilities
- **Fallback Systems**: Triple redundancy (batch → individual → local) ensures reliability

### Database Constraints
- SQLite is used for simplicity and portability
- FSRS statistics must be atomically updated with review submissions
- Card content supports LaTeX/MathJax markup for mathematical expressions (both inline `$...$` and display `$$...$$`)
- Search functionality uses case-insensitive LIKE queries for content matching
- Backlinks maintain referential integrity through CASCADE DELETE foreign key constraints

### Frontend Architecture
- **Design System**: CSS custom properties for consistent spacing, typography, and colors
- **Component Structure**: Modular card preview system, skeleton loading states, toast notifications
- **State Management**: JavaScript-based with real-time search debouncing and progress tracking
- **Icon System**: Feather Icons with SVG sprites for scalable, accessible iconography
- **Responsive Strategy**: Mobile-first CSS with progressive enhancement for larger screens

## Project Status
**Phase 3 Complete**: Quiz efficiency optimizations successfully implemented alongside modern UI/UX enhancements. The system now features state-of-the-art performance optimizations with:

### ✅ Completed Features
- **Navigation**: Icon-enhanced navigation with Feather Icons (layers, refresh-cw, tag icons)
- **Search**: Real-time search with debouncing, highlighting, and case-insensitive matching  
- **Cards**: Preview system for long content with expand/collapse functionality
- **Linking**: Bidirectional backlinks with automatic maintenance and distinct visual styling
- **Math Rendering**: Full LaTeX support with inline `$...$` and display `$$...$$` math
- **Reviews**: Progress tracking, keyboard shortcuts (Space, 1-4), and completion celebrations
- **Responsive**: Mobile-first design with touch-friendly interactions
- **Feedback**: Toast notifications, skeleton loading, and enhanced error handling
- **Accessibility**: High contrast mode, reduced motion, semantic HTML
- **Efficiency**: Batch processing for 85-90% reduction in API calls per session
- **Smart Ordering**: Multi-factor card optimization for better LLM context
- **Performance Monitoring**: Comprehensive structured logging and error tracking
- **Centralized Logging**: File-based logging with daily rotation in `logs/learning-system.log.YYYY-MM-DD`
- **Session Answer Submission**: Context-aware grading for multiple choice questions with proper question context

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
- **Database Tests**: Backlink CRUD operations, foreign key constraints, cascade deletions
- **UI Feature Tests**: Navigation icons, loading states, responsive design validation
- **Error Handling**: Edge cases, concurrent operations, invalid data handling, non-existent link targets
- **Efficiency Tests**: Batch processing, smart ordering algorithms, fallback mechanisms
- **Performance Tests**: API call reduction, content bucketing, overdue ratio calculations
- **Session Answer Tests**: Context-aware answer submission and multiple choice grading validation

### Development Notes
- All Phase 2 UI/UX improvements from UI_IMPROVEMENTS.md completed
- All Phase 3 efficiency optimizations from QUIZ_EFFICIENCY_IMPROVEMENTS.md completed
- Bidirectional backlinks system fully implemented and tested
- LaTeX rendering supports both inline and display math expressions
- Batch processing infrastructure with robust fallback mechanisms
- Smart card ordering with multi-factor optimization algorithms
- Comprehensive structured logging throughout the system
- Session-based answer submission fixes multiple choice grading issue
- Legacy endpoint cleanup removes unused and problematic API routes
- Design system provides consistent foundation for future development
- Mobile-first responsive approach supports modern device usage patterns
- 113 total tests covering all functionality including session answer submission

### Performance Achievements
- **API Call Reduction**: 85-90% fewer calls per review session (from ~15-20 to 1-2 calls)
- **Session Initialization**: Instant question access through batch pre-generation
- **Smart Ordering**: Optimized card sequence for better LLM context utilization
- **Context-Aware Grading**: Multiple choice questions graded against actual question context
- **Monitoring**: Comprehensive structured logging for performance visibility

**Latest Update**: Legacy endpoint cleanup completed - frontend migrated to session-based answer submission, unused endpoints removed
**Next Phase**: Ready for Phase 4 advanced features (session persistence, dark mode, advanced study statistics)