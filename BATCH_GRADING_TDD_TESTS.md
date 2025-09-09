# Batch Grading TDD Test Suite

## Overview

This document describes the comprehensive Test-Driven Development (TDD) test suite created for the parallel batch grading implementation. The tests are designed to validate the complete Phase 1 implementation before any code is written.

## Test Structure

### 1. API Layer Tests (`src/tests/batch_grading_test.rs`)

**Purpose**: Validate the new batch grading API endpoint functionality

**Key Test Cases**:
- ✅ **Successful batch submission** - Validates complete request/response flow
- ✅ **Invalid session handling** - Ensures proper 404 response for non-existent sessions
- ✅ **Invalid card handling** - Validates card existence checking
- ✅ **Invalid question index** - Tests bounds checking for question arrays
- ✅ **Empty request validation** - Handles empty answer arrays
- ✅ **Mixed correctness grading** - Tests both correct and incorrect answers
- ✅ **Performance validation** - Ensures batch processing completes in reasonable time
- ✅ **Response format consistency** - Validates JSON structure matches specification
- ✅ **Answer ordering preservation** - Ensures results maintain input order

**Expected Endpoint**: `POST /api/review/session/:session_id/answers/:card_id/batch`

**Request Format**:
```json
{
  "answers": [
    {"question_index": 0, "answer": "user answer"},
    {"question_index": 1, "answer": "another answer"}
  ]
}
```

**Response Format**:
```json
{
  "success": true,
  "data": [
    {
      "question_id": "1",
      "is_correct": true,
      "feedback": "Feedback text",
      "suggested_rating": 4
    }
  ]
}
```

### 2. Service Layer Tests (`src/tests/batch_grading_service_test.rs`)

**Purpose**: Validate the LLM service batch grading logic and data transformations

**Key Test Cases**:
- ✅ **Batch request construction** - Validates BatchGradingRequest building from inputs
- ✅ **Batch grading execution** - Tests the core batch grading workflow
- ✅ **Empty request handling** - Validates handling of empty request arrays
- ✅ **Single request processing** - Ensures single-item batches work correctly
- ✅ **Mixed question types** - Tests both multiple choice and short answer questions
- ✅ **Incorrect answer grading** - Validates low ratings for wrong answers
- ✅ **Result ordering** - Ensures results maintain question order
- ✅ **Long content truncation** - Tests content length limits (300 chars)
- ✅ **Fallback mechanism** - Validates individual grading fallback on batch failure
- ✅ **Large batch processing** - Tests scalability with 20+ questions

**Service Method Under Test**: `LLMService::grade_batch_answers()`

### 3. Frontend Tests (`static/test-batch-grading.html`)

**Purpose**: Validate frontend batch submission logic and user interaction

**Key Test Cases**:
- ✅ **Batch submission with valid answers** - Tests successful submission flow
- ✅ **Batch request structure validation** - Validates correct API request format
- ✅ **Missing answers validation** - Prevents submission with incomplete answers
- ✅ **API error handling** - Graceful handling of server errors
- ✅ **Mixed question types handling** - Supports both multiple choice and short answer
- ✅ **Performance simulation** - Demonstrates batch vs sequential timing
- ✅ **Answer ordering preservation** - Maintains question-answer relationships
- ✅ **Empty quiz handling** - Handles edge case of no questions

**Frontend Method Under Test**: `submitAllAnswersBatch()`

**Features**:
- Interactive browser-based test runner
- Mock API responses for isolated testing
- Visual pass/fail indicators
- Performance timing comparisons
- Detailed error reporting

### 4. Integration Tests (`src/tests/batch_grading_integration_test.rs`)

**Purpose**: Validate end-to-end workflow from API request to database update

**Key Test Cases**:
- ✅ **Complete batch grading workflow** - Full request → grading → response cycle
- ✅ **Batch vs sequential performance comparison** - Validates performance improvements
- ✅ **Error recovery mechanisms** - Tests graceful degradation
- ✅ **Large content handling** - Validates processing of long answers (1000+ chars)
- ✅ **Concurrent batch requests** - Tests system under concurrent load
- ✅ **Session state consistency** - Ensures session data remains valid
- ✅ **End-to-end review workflow** - Complete user journey validation

**Performance Requirements**:
- Batch processing must be ≥1.5x faster than sequential
- Requests must complete within 1 second for typical batches
- Concurrent requests must not interfere with each other

## Mock Services Required

### LLMService Mock Methods

```rust
impl LLMService {
    fn new_mock() -> Self;
    fn new_mock_with_incorrect_answers() -> Self;
    fn new_mock_with_batch_failure() -> Self;
}
```

### CardService Mock Methods

```rust
impl CardService {
    async fn new_in_memory() -> Result<Self>;
}
```

## Test Data Specifications

### Test Card
```rust
Card {
    id: Uuid::new_v4(),
    zettel_id: "TEST001",
    title: Some("Test Card"),
    content: "Test content for batch grading",
    // ... other fields
}
```

### Test Questions
```rust
vec![
    QuizQuestion {
        question: "Short answer question",
        question_type: "short_answer",
        correct_answer: Some("Expected answer"),
    },
    QuizQuestion {
        question: "Multiple choice question", 
        question_type: "multiple_choice",
        options: Some(vec!["A) Option 1", "B) Option 2"]),
        correct_answer: Some("A) Option 1"),
    }
]
```

## Success Criteria

### API Tests Must Pass
- [ ] All HTTP status codes correct (200, 400, 404)
- [ ] JSON response structure matches specification
- [ ] Error handling works for all edge cases
- [ ] Request validation prevents invalid submissions

### Service Tests Must Pass  
- [ ] Batch grading produces correct results
- [ ] Fallback mechanisms work when batch fails
- [ ] Performance within acceptable limits
- [ ] Data transformations preserve integrity

### Frontend Tests Must Pass
- [ ] User interactions create correct API requests
- [ ] UI handles success and error states properly
- [ ] Answer collection works for all question types
- [ ] Validation prevents incomplete submissions

### Integration Tests Must Pass
- [ ] Complete workflow from UI to database works
- [ ] Performance improvements are measurable (≥1.5x)
- [ ] Error recovery doesn't corrupt state
- [ ] Concurrent usage is stable

## Running the Tests

### Backend Tests
```bash
# Run all batch grading tests
cargo test batch_grading

# Run specific test modules
cargo test batch_grading_test
cargo test batch_grading_service_test  
cargo test batch_grading_integration_test
```

### Frontend Tests
```bash
# Start development server
cargo run

# Open in browser
http://localhost:3000/test-batch-grading.html

# Click "Run All Tests" button
```

### Full Test Suite
```bash
# Backend tests
cargo test batch_grading

# Manual frontend validation
# Navigate to test-batch-grading.html and verify all tests pass
```

## Implementation Guidance

### TDD Workflow
1. **Red**: Run tests - they should fail initially
2. **Green**: Implement minimum code to make tests pass  
3. **Refactor**: Improve code while keeping tests passing
4. **Repeat**: Add more functionality guided by failing tests

### Implementation Order
1. Create mock services to support test infrastructure
2. Implement BatchAnswerRequest/Response types
3. Create batch API endpoint handler
4. Update frontend submitAllAnswersBatch method
5. Add route registration
6. Optimize and refactor based on test feedback

### Test-First Benefits
- **Specification clarity**: Tests define exact expected behavior
- **Regression protection**: Changes can't break existing functionality  
- **Design validation**: Tests verify API design before implementation
- **Documentation**: Tests serve as executable documentation
- **Confidence**: Full test coverage ensures robust implementation

## Next Steps

1. Run initial tests to confirm they fail appropriately
2. Begin implementation following TDD red-green-refactor cycle
3. Use test failures to guide development priorities
4. Validate performance improvements with integration tests
5. Update documentation based on test learnings

This TDD approach ensures the batch grading implementation will be robust, well-tested, and meet all specified requirements from day one.