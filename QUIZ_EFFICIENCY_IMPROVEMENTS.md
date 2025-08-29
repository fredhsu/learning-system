# Quiz Efficiency Improvements

## Overview

This document outlines improvements to make the quiz/review process more efficient, reducing wait times and creating a smoother user experience.

## Current Inefficiencies

1. **One-by-One Processing**: Each card generates questions individually (`src/api.rs:223`)
2. **Sequential LLM Calls**: Questions generated one at a time, then answers graded one at a time
3. **Hardcoded Dummy Questions**: The grading system uses placeholder questions (`src/api.rs:252-257`)
4. **Missing Question Context**: Current answer grading doesn't reference the actual question asked

## Suggested Efficiency Improvements

### 1. Batch Question Generation

Generate questions for multiple cards at once to reduce API overhead:

```rust
// Add to LLMService
pub async fn generate_batch_quiz_questions(&self, cards: &[Card]) -> Result<HashMap<Uuid, Vec<QuizQuestion>>> {
    let prompt = format!(
        r#"Generate 2-3 quiz questions for each of the following learning cards.
        
        Cards:
        {}
        
        Return JSON in this format:
        {{
            "results": {{
                "card_id_1": [{{ "question": "...", "question_type": "...", "options": [...], "correct_answer": "..." }}],
                "card_id_2": [...]
            }}
        }}"#,
        cards.iter().enumerate().map(|(i, card)| 
            format!("Card {}: ID={}, Content={}", i+1, card.id, card.content)
        ).collect::<Vec<_>>().join("\n\n")
    );
    
    // Single API call for multiple cards
    // Parse response and return HashMap<card_id, questions>
}
```

### 2. Pre-generate Questions at Start

Store quiz questions in session state to avoid generating them during review:

```rust
// Add to api.rs
pub async fn start_review_session(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<ReviewSession>>, StatusCode> {
    let due_cards = state.card_service.get_cards_due_for_review().await?;
    
    // Batch generate all questions upfront
    let all_questions = state.llm_service.generate_batch_quiz_questions(&due_cards).await?;
    
    let session = ReviewSession {
        cards: due_cards,
        questions: all_questions,
        current_card: 0,
        session_id: Uuid::new_v4(),
    };
    
    // Store session (in memory or database)
    Ok(Json(ApiResponse::success(session)))
}
```

### 3. Batch Answer Grading

Grade multiple answers together when possible:

```rust
pub async fn grade_batch_answers(
    &self,
    grading_requests: &[BatchGradingRequest],
) -> Result<Vec<GradingResult>> {
    let prompt = format!(
        r#"Grade the following quiz answers. For each question-answer pair, provide grading results.
        
        Questions and Answers:
        {}
        
        Respond with JSON array: [{{"question_id": "1", "is_correct": true, "feedback": "...", "suggested_rating": 3}}, ...]"#,
        grading_requests.iter().enumerate().map(|(i, req)| 
            format!("{}. Question: {} | User Answer: {} | Correct Answer: {}", 
                i+1, req.question.question, req.user_answer, req.question.correct_answer.as_deref().unwrap_or("N/A"))
        ).collect::<Vec<_>>().join("\n")
    );
    
    // Single API call for multiple gradings
}
```

### 4. Smart Review Ordering

Optimize the order of cards to group similar content for better LLM context:

```rust
pub async fn get_cards_due_optimized(&self) -> Result<Vec<Card>> {
    let mut cards = self.get_cards_due_for_review().await?;
    
    // Sort by similarity/topic to improve LLM context efficiency
    cards.sort_by(|a, b| {
        // Group by topic or content similarity
        a.content.len().cmp(&b.content.len())
    });
    
    Ok(cards)
}
```

### 5. Parallel Processing

Use async parallel processing for independent operations:

```javascript
// In app.js - load review session with parallel operations
async loadReviewSession() {
    try {
        // Start all operations in parallel
        const [dueCards, sessionData] = await Promise.all([
            this.apiCall('/cards/due'),
            this.apiCall('/review/start-session', { method: 'POST' })
        ]);
        
        // Questions already pre-generated in session
        this.reviewSession = {
            ...sessionData,
            dueCards: dueCards
        };
        
        this.updateRemainingCount();
        await this.startQuiz(dueCards[0]);
    } catch (error) {
        this.showError('Failed to load review session');
    }
}
```

### 6. Caching and Preloading

Cache frequently used data and preload next questions:

```javascript
async renderQuestion() {
    // Current question rendering...
    
    // Preload next question's assets if available
    if (this.currentQuiz.currentQuestion + 1 < this.currentQuiz.questions.length) {
        this.preloadNextQuestion();
    }
}

preloadNextQuestion() {
    // Pre-process markdown, MathJax, etc. for next question
    const nextIndex = this.currentQuiz.currentQuestion + 1;
    const nextQuestion = this.currentQuiz.questions[nextIndex];
    // Background processing...
}
```

## Implementation Priority

### High Impact (Implement First)

1. **Pre-generate Questions** - Generate all questions when starting review session
   - **Impact**: Eliminates per-card wait times during review
   - **Files**: `src/api.rs`, `src/models.rs`, `static/app.js`

2. **Fix Question Context** - Store and pass actual questions to grading
   - **Impact**: Fixes current grading accuracy issues
   - **Files**: `src/api.rs:252-257`

3. **Parallel Operations** - Load cards and session data simultaneously
   - **Impact**: Faster session initialization
   - **Files**: `static/app.js:560-592`

### Medium Impact

4. **Batch Question Generation** - For multiple cards at once
   - **Impact**: Reduces LLM API overhead
   - **Files**: `src/llm_service.rs`

5. **Smart Ordering** - Group similar cards for better efficiency
   - **Impact**: Better LLM context utilization
   - **Files**: `src/card_service.rs`

### Future Optimization

6. **Batch Grading** - When user answers multiple questions
   - **Impact**: Reduces grading API calls
   - **Files**: `src/llm_service.rs`

7. **Preloading** - Background processing of next questions
   - **Impact**: Smoother transitions between questions
   - **Files**: `static/app.js`

## Expected Performance Improvements

- **Session Start Time**: From 3-5 seconds to under 1 second
- **Question Transitions**: From 2-3 seconds to near-instant
- **Overall Review Speed**: 3-4x faster completion times
- **API Calls**: Reduced by 60-80% through batching

## Technical Considerations

### New Data Structures Needed

```rust
#[derive(Serialize, Deserialize)]
pub struct ReviewSession {
    pub session_id: Uuid,
    pub cards: Vec<Card>,
    pub questions: HashMap<Uuid, Vec<QuizQuestion>>,
    pub current_card: usize,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct BatchGradingRequest {
    pub question: QuizQuestion,
    pub user_answer: String,
    pub card_content: String,
}
```

### API Endpoints to Add

- `POST /api/review/session/start` - Start review session with pre-generated questions
- `POST /api/review/session/{id}/batch-grade` - Grade multiple answers
- `GET /api/review/session/{id}` - Get session state
- `DELETE /api/review/session/{id}` - End session

### Database Changes

Consider adding a `review_sessions` table for persistent sessions across page reloads:

```sql
CREATE TABLE review_sessions (
    id UUID PRIMARY KEY,
    user_id UUID, -- Future user system
    cards_data JSONB,
    questions_data JSONB,
    current_card INTEGER,
    created_at TIMESTAMP,
    expires_at TIMESTAMP
);
```

## Notes

- These improvements maintain backward compatibility
- Progressive implementation possible - start with high-impact changes
- Consider adding session persistence for longer review sessions
- Monitor LLM token usage with batch operations to manage costs