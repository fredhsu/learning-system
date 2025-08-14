# Knowledge Management + Spaced Repetition Quiz System

## Goal
Build a knowledge management and spaced repetition learning platform using the FSRS algorithm, with a web-based frontend and LLM-powered quiz generation/grading.

## Workflow
1. **Scheduling**: FSRS (Rust) determines topics/cards due for review.
2. **Question Generation**: Relevant cards are sent to an LLM with a structured prompt to generate quiz questions (multiple choice, short answer, problem-solving).
3. **User Interaction**: User answers questions in the web UI.
4. **Grading**: LLM grades answers based on predefined rubrics.
5. **Scheduling Update**: Backend updates FSRS statistics and schedules next review.

## Core Features
- Cards stored in a SQLite database with wiki-style links.
- Cards can belong to one or more topics.
- Zettelkasten-inspired linking and organization.
- LaTeX/MathJax support for math/science content.
- LLM prompts for question generation and grading with JSON input/output.
- Review history tracking.

## Database Schema (Example)
**cards**: id, content, creation_date, last_reviewed, next_review, difficulty_rating, links
**topics**: id, name, description
**card_topics**: card_id, topic_id
**reviews**: id, card_id, review_date, score, interval, ease_factor

## Technical Requirements
- Backend in Rust.
- FSRS via [rs-fsrs](https://github.com/open-spaced-repetition/rs-fsrs).
- SQLite database.
- REST or GraphQL API between backend and frontend.
- Frontend: SPA or SSR with MathJax preview, rich text editor, and mobile-friendly UI.

## Development Steps
1. **Database Schema & Models** – Implement schema with support for FSRS stats and card linking.
2. **FSRS Integration** – Connect Rust backend with FSRS library for scheduling.
3. **Card Management** – CRUD operations for cards/topics and linking.
4. **LLM Integration** – Prompt templates for quiz generation and grading with JSON outputs.
5. **Review Flow** – Web UI for quizzes, answer submission, and feedback.
6. **Scheduling Updates** – Automatic FSRS updates post-grading.
7. **Testing** – Unit tests, integration tests, mock data for UI testing.

