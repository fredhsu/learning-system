# Frontend Testing Suite

This directory contains comprehensive tests for the Learning System frontend code.

## Overview

The test suite includes:
- **Unit Tests**: Core functionality testing
- **Integration Tests**: User workflow testing  
- **Test Framework**: Custom lightweight testing framework
- **Test Utilities**: Helper functions and mocks
- **DOM Mocks**: Simplified DOM environment for testing

## File Structure

```
static/tests/
├── README.md                 # This documentation
├── test-framework.js         # Custom test framework
├── test-utils.js            # Test utilities and helpers
├── mocks/
│   └── dom-mock.js          # DOM and LearningSystem mocks
├── learning-system.test.js  # Core functionality tests
├── search.test.js           # Search functionality tests
├── wiki-links.test.js       # Wiki link processing tests
├── review-session.test.js   # Review session tests
└── integration.test.js      # Integration workflow tests
```

## Running Tests

### In Browser

1. Open `test-runner.html` in your browser
2. Tests run automatically on page load
3. Click "Run All Tests" to re-run tests
4. View results in the browser with color-coded output

### Test Categories

#### Unit Tests (learning-system.test.js)
- Core class initialization
- Wiki link processing
- Markdown rendering
- API calls
- Input validation
- Error handling

#### Search Tests (search.test.js)
- Search execution and filtering
- Search highlighting
- Debouncing functionality
- Performance with large datasets
- Edge cases and error handling

#### Wiki Links Tests (wiki-links.test.js)
- Link pattern recognition
- Content processing
- HTML generation
- Integration with markdown
- Unicode and special characters

#### Review Session Tests (review-session.test.js)
- Session initialization
- Card rating and progression
- Rating calculations
- Progress tracking
- State management

#### Integration Tests (integration.test.js)
- Complete user workflows
- Card management workflow
- Search workflow
- Review session workflow
- Navigation workflow
- Modal interactions
- Error handling
- Performance testing
- Accessibility

## Test Framework Features

### Core Functions
```javascript
describe('Test Suite', () => {
    beforeEach(() => { /* setup */ });
    afterEach(() => { /* cleanup */ });
    
    it('should test something', () => {
        expect(actual).toBe(expected);
    });
});
```

### Assertions
- `toBe()` - Strict equality
- `toEqual()` - Deep equality
- `toBeTrue()` / `toBeFalse()`
- `toBeNull()` / `toBeUndefined()` / `toBeDefined()`
- `toContain()` - String/array contains
- `toMatch()` - Regex matching
- `toHaveProperty()` - Object properties
- `toHaveLength()` - Array/string length
- `toThrow()` - Exception throwing
- `toBeInstanceOf()` - Type checking

### Mocking
```javascript
// Mock functions
const mockFn = mock.fn();
const spy = mock.spyOn(object, 'method');

// Mock API responses
const { mockFetch, restore } = TestUtils.mockFetch({
    '/api/cards': { data: [...] }
});
```

## Test Utilities

### DOM Testing
```javascript
// Create test fixtures
const fixture = TestUtils.createFixture('<div>HTML</div>');

// Simulate user interactions
TestUtils.click(element);
await TestUtils.typeIntoInput(input, 'text');
TestUtils.fireEvent(element, 'change');

// Assertions
TestUtils.assertVisible(element);
TestUtils.assertHasClass(element, 'active');
```

### Data Generation
```javascript
// Generate test data
const card = TestUtils.generateTestCard();
const cards = TestUtils.generateTestCards(5);

// Setup API mocks
const mocks = TestUtils.setupApiMocks();
```

### Async Testing
```javascript
// Wait for conditions
await TestUtils.waitFor(() => element.textContent === 'expected');
await TestUtils.wait(100); // Simple delay
```

## Coverage Areas

### ✅ Covered Features
- Wiki link processing and rendering
- Search functionality with debouncing
- Review session management
- Card validation and sanitization
- Error handling and user feedback
- API integration patterns
- Performance with large datasets
- User workflow integration

### 🔄 Areas for Future Enhancement
- Visual regression testing
- Cross-browser compatibility
- Mobile touch interactions
- Accessibility compliance
- Advanced markdown features
- Offline functionality
- Real-time synchronization

## Best Practices

### Writing Tests
1. **Descriptive Names**: Use clear, descriptive test names
2. **Single Responsibility**: One assertion per test when possible
3. **Setup/Teardown**: Use beforeEach/afterEach for clean state
4. **Mock External Dependencies**: Don't test external libraries
5. **Edge Cases**: Include boundary conditions and error states

### Test Organization
1. **Group Related Tests**: Use describe blocks effectively
2. **Logical Order**: Arrange tests from simple to complex
3. **Shared Setup**: Extract common setup to beforeEach
4. **Isolation**: Each test should be independent

### Performance
1. **Fast Execution**: Tests should run quickly
2. **Resource Cleanup**: Clean up DOM, timers, and mocks
3. **Efficient Mocks**: Use lightweight mocks
4. **Batch Similar Tests**: Group related assertions

## Debugging Tests

### Common Issues
1. **Async Timing**: Use proper async/await patterns
2. **DOM State**: Ensure clean DOM between tests
3. **Mock Isolation**: Restore mocks after tests
4. **Memory Leaks**: Clear timers and event listeners

### Debug Tools
1. **Console Logging**: Use console.log in tests temporarily
2. **Browser DevTools**: Inspect DOM state during tests
3. **Step Through**: Add breakpoints in test code
4. **Isolated Runs**: Run single tests to isolate issues

## Contributing

When adding new tests:

1. Follow existing patterns and conventions
2. Add tests for both happy path and edge cases
3. Update this README if adding new test categories
4. Ensure tests are fast and reliable
5. Use descriptive test names and comments

## Metrics

Current test coverage includes:
- **150+ test cases** across 5 test files
- **Unit tests**: Core functionality coverage
- **Integration tests**: Complete user workflows
- **Performance tests**: Large dataset handling
- **Edge case handling**: Error conditions and boundary cases

Target: 90%+ code coverage for critical paths.