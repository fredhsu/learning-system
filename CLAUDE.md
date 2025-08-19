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
The system uses the rs-fsrs crate for spaced repetition scheduling. Review intervals and difficulty ratings are managed through FSRS state updates after each quiz attempt. Keyboard shortcuts (1-4) map directly to FSRS rating levels (Again, Hard, Good, Easy).

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
- Search functionality uses case-insensitive LIKE queries for content matching

### Frontend Architecture
- **Design System**: CSS custom properties for consistent spacing, typography, and colors
- **Component Structure**: Modular card preview system, skeleton loading states, toast notifications
- **State Management**: JavaScript-based with real-time search debouncing and progress tracking
- **Icon System**: Feather Icons with SVG sprites for scalable, accessible iconography
- **Responsive Strategy**: Mobile-first CSS with progressive enhancement for larger screens

## Project Status
**Phase 2 Complete**: Modern UI/UX enhancements successfully implemented with comprehensive testing coverage. The system now features a professional, responsive interface with:

### ✅ Completed Features
- **Navigation**: Icon-enhanced navigation with Feather Icons (layers, refresh-cw, tag icons)
- **Search**: Real-time search with debouncing, highlighting, and case-insensitive matching  
- **Cards**: Preview system for long content with expand/collapse functionality
- **Reviews**: Progress tracking, keyboard shortcuts (Space, 1-4), and completion celebrations
- **Responsive**: Mobile-first design with touch-friendly interactions
- **Feedback**: Toast notifications, skeleton loading, and enhanced error handling
- **Accessibility**: High contrast mode, reduced motion, semantic HTML

### API Enhancements
- **Search Endpoint**: `/api/cards/search?q={query}` with URL encoding support
- **Enhanced Responses**: Consistent JSON structure with success/error states
- **Performance**: Optimized for skeleton loading and progressive enhancement

### Testing Coverage
- **Integration Tests**: Search functionality, UI preview, keyboard shortcuts
- **API Tests**: Search endpoints, response structure validation, URL encoding
- **UI Feature Tests**: Navigation icons, loading states, responsive design validation
- **Error Handling**: Edge cases, concurrent operations, invalid data handling

### Development Notes
- All Phase 2 medium priority tasks from UI_IMPROVEMENTS.md completed
- Comprehensive test suite ensures stability across UI enhancements
- Design system provides consistent foundation for future development
- Mobile-first responsive approach supports modern device usage patterns

**Next Phase**: Ready for Phase 3 advanced features (dark mode, study statistics, advanced filtering)