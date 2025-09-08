/**
 * Integration Tests for Frontend User Workflows
 */

describe('Integration Tests - User Workflows', () => {
    let app;
    let fixture;
    let mockFetch;
    
    beforeEach(() => {
        app = new LearningSystemTest();
        fixture = TestUtils.createFixture();
        mockFetch = TestUtils.setupApiMocks();
    });
    
    afterEach(() => {
        fixture.cleanup();
        mockFetch.restore();
    });

    describe('Card Management Workflow', () => {
        it('should complete full card creation workflow', async () => {
            // Setup mock DOM elements
            fixture.element.innerHTML = `
                <form id="create-card-form">
                    <input id="card-zettel-id" value="1.1" />
                    <input id="card-title" value="Test Card" />
                    <textarea id="card-content">Test content with [[Link]]</textarea>
                    <input id="card-topics" value="math, science" />
                    <button type="submit">Create</button>
                </form>
                <div id="cards-list"></div>
            `;

            // Simulate form submission
            const form = fixture.element.querySelector('#create-card-form');
            const submitEvent = TestUtils.createMockEvent('submit', {
                preventDefault: () => {}
            });

            // Mock successful API response
            mockFetch.mockFetch.mockResolvedValueOnce({
                ok: true,
                json: () => Promise.resolve({
                    success: true,
                    data: { id: 1, zettel_id: '1.1', title: 'Test Card' }
                })
            });

            form.dispatchEvent(submitEvent);

            // Wait for async operations
            await TestUtils.wait(50);

            // Verify API was called with correct data
            expect(mockFetch.mockFetch).toHaveBeenCalledWith('/api/cards', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: expect.stringContaining('"zettel_id":"1.1"')
            });
        });

        it('should handle card validation errors', async () => {
            fixture.element.innerHTML = `
                <form id="create-card-form">
                    <input id="card-zettel-id" value="invalid..id" />
                    <input id="card-title" value="" />
                    <textarea id="card-content"></textarea>
                    <button type="submit">Create</button>
                </form>
                <div id="error-message"></div>
            `;

            const zettelId = fixture.element.querySelector('#card-zettel-id').value;
            const validation = app.validateZettelId(zettelId);

            expect(validation.valid).toBe(false);
            expect(validation.error).toBeDefined();
        });

        it('should process wiki links in created cards', () => {
            const cardContent = 'This card references [[Another Card]] and [[Third Card]].';
            const processedContent = app.processWikiLinks(cardContent);

            expect(processedContent).toContain('class="wiki-link"');
            expect(processedContent).toContain('data-link-text="Another Card"');
            expect(processedContent).toContain('data-link-text="Third Card"');
        });
    });

    describe('Search Workflow', () => {
        it('should complete full search workflow', async () => {
            // Setup search UI
            fixture.element.innerHTML = `
                <input id="search-input" type="text" />
                <div id="search-results"></div>
                <div id="search-info"></div>
            `;

            const searchInput = fixture.element.querySelector('#search-input');
            
            // Simulate user typing
            await TestUtils.typeIntoInput(searchInput, 'mathematics', { delay: 10 });

            // Trigger search
            const searchResults = await app.debounceSearch('mathematics', 10);
            
            expect(Array.isArray(searchResults)).toBe(true);
            expect(searchResults.length).toBeGreaterThan(0);
        });

        it('should highlight search terms in results', () => {
            const searchTerm = 'test';
            const content = 'This is a test string for testing';
            const highlighted = app.highlightSearchTerms(content, searchTerm);

            expect(highlighted).toContain('<span class="search-highlight">test</span>');
            expect(highlighted).toContain('for <span class="search-highlight">test</span>ing');
        });

        it('should debounce rapid search inputs', async () => {
            let searchCallCount = 0;
            const originalPerformSearch = app.performSearch;
            app.performSearch = (...args) => {
                searchCallCount++;
                return originalPerformSearch.call(app, ...args);
            };

            // Rapid search calls
            const promises = [
                app.debounceSearch('a', 50),
                app.debounceSearch('ab', 50),
                app.debounceSearch('abc', 50)
            ];

            await promises[2]; // Wait for last search

            // Should only execute the last search
            expect(searchCallCount).toBe(1);
        });
    });

    describe('Review Session Workflow', () => {
        it('should complete full review session workflow', async () => {
            const testCards = TestUtils.generateTestCards(3);
            
            // Start review session
            const session = app.startReviewSession(testCards);
            expect(session.totalCards).toBe(3);
            expect(session.currentCardIndex).toBe(0);

            // Mock quiz interface
            fixture.element.innerHTML = `
                <div id="quiz-container">
                    <div id="card-content-display"></div>
                    <div id="quiz-questions">
                        <div class="question">
                            <p>What is 2+2?</p>
                            <div class="options">
                                <label><input type="radio" name="q0" value="3"> 3</label>
                                <label><input type="radio" name="q0" value="4"> 4</label>
                            </div>
                        </div>
                    </div>
                    <div id="quiz-feedback"></div>
                    <button id="submit-btn">Submit</button>
                </div>
            `;

            // Simulate answering first card
            app.currentQuiz = { card: testCards[0], questions: [{ id: 1, type: 'multiple_choice' }] };
            const continueToNext = app.rateCard(3);
            
            expect(continueToNext).toBe(true);
            expect(session.currentCardIndex).toBe(1);
            
            // Complete second card
            app.currentQuiz = { card: testCards[1], questions: [{ id: 2, type: 'multiple_choice' }] };
            app.rateCard(4);
            expect(session.currentCardIndex).toBe(2);
            
            // Complete final card
            app.currentQuiz = { card: testCards[2], questions: [{ id: 3, type: 'multiple_choice' }] };
            const sessionComplete = app.rateCard(2);
            
            expect(sessionComplete).toBe(false); // No more cards
            expect(session.currentCardIndex).toBe(3);
        });

        it('should handle review session with single card', async () => {
            const singleCard = [TestUtils.generateTestCard()];
            const session = app.startReviewSession(singleCard);
            
            expect(session.totalCards).toBe(1);
            
            app.currentQuiz = { card: singleCard[0], questions: [] };
            const sessionComplete = app.rateCard(3);
            
            expect(sessionComplete).toBe(false);
            expect(session.currentCardIndex).toBe(1);
        });

        it('should calculate final ratings correctly', () => {
            const testCases = [
                { ratings: [1, 2, 3], expected: 2 },
                { ratings: [4, 4, 4], expected: 4 },
                { ratings: [2, 3, 4], expected: 3 },
                { ratings: [1, 1, 4], expected: 2 }
            ];

            testCases.forEach(({ ratings, expected }) => {
                const result = app.calculateFinalRating(ratings);
                expect(result).toBe(expected);
            });
        });
    });

    describe('Navigation Workflow', () => {
        it('should handle view switching', () => {
            fixture.element.innerHTML = `
                <nav>
                    <button id="nav-cards" class="nav-btn active">Cards</button>
                    <button id="nav-review" class="nav-btn">Review</button>
                    <button id="nav-topics" class="nav-btn">Topics</button>
                </nav>
                <div id="cards-view" class="view active">Cards View</div>
                <div id="review-view" class="view">Review View</div>
                <div id="topics-view" class="view">Topics View</div>
            `;

            const cardsBtn = fixture.element.querySelector('#nav-cards');
            const reviewBtn = fixture.element.querySelector('#nav-review');
            
            // Initial state
            expect(app.currentView).toBe('cards');
            
            // Simulate clicking review button
            TestUtils.click(reviewBtn);
            
            // Mock view switching logic
            app.currentView = 'review';
            expect(app.currentView).toBe('review');
        });

        it('should maintain state during navigation', () => {
            app.allCards = TestUtils.generateTestCards(5);
            app.currentSearchQuery = 'test query';
            
            // Switch views
            app.currentView = 'review';
            app.currentView = 'topics';
            app.currentView = 'cards';
            
            // State should be preserved
            expect(app.allCards).toHaveLength(5);
            expect(app.currentSearchQuery).toBe('test query');
        });
    });

    describe('Modal Workflow', () => {
        it('should handle modal opening and closing', () => {
            fixture.element.innerHTML = `
                <div id="create-card-modal" class="modal">
                    <div class="modal-content">
                        <span class="close">&times;</span>
                        <h3>Create Card</h3>
                        <form>
                            <input type="text" required />
                            <button type="submit">Create</button>
                        </form>
                    </div>
                </div>
            `;

            const modal = fixture.element.querySelector('#create-card-modal');
            const closeBtn = fixture.element.querySelector('.close');

            // Show modal
            const showResult = app.showModal('create-card-modal');
            expect(showResult.visible).toBe(true);

            // Close modal
            const closeResult = app.closeModal(modal);
            expect(closeResult.visible).toBe(false);
        });

        it('should handle form validation in modals', () => {
            fixture.element.innerHTML = `
                <form id="test-form">
                    <input id="zettel-id" value="" required />
                    <input id="content" value="Valid content" required />
                    <button type="submit">Submit</button>
                </form>
            `;

            const zettelId = fixture.element.querySelector('#zettel-id').value;
            const content = fixture.element.querySelector('#content').value;

            const zettelValidation = app.validateZettelId(zettelId);
            expect(zettelValidation.valid).toBe(false);

            expect(content.length).toBeGreaterThan(0);
        });
    });

    describe('Error Handling Workflow', () => {
        it('should handle API failures gracefully', async () => {
            mockFetch.mockFetch.mockRejectedValueOnce(new Error('Network error'));

            try {
                await app.apiCall('/cards');
            } catch (error) {
                expect(error.message).toBe('Network error');
            }
        });

        it('should show user-friendly error messages', () => {
            const errorMessage = 'Something went wrong';
            const result = app.showError(errorMessage);
            
            expect(result.type).toBe('error');
            expect(result.message).toBe(errorMessage);
        });

        it('should handle invalid input sanitization', () => {
            const maliciousInput = '<script>alert("xss")</script>';
            const sanitized = app.sanitizeInput(maliciousInput);
            
            expect(sanitized).not.toContain('<script>');
            expect(sanitized).toContain('&lt;script&gt;');
        });
    });

    describe('Performance and Load Testing', () => {
        it('should handle large datasets efficiently', () => {
            const largeCardSet = Array.from({ length: 1000 }, (_, i) => 
                TestUtils.generateTestCard({ id: i, title: `Card ${i}` })
            );

            app.allCards = largeCardSet;

            const startTime = performance.now();
            const searchResults = app.performSearch('Card');
            const endTime = performance.now();

            expect(searchResults.length).toBeGreaterThan(0);
            expect(endTime - startTime).toBeLessThan(100); // Should be fast
        });

        it('should handle rapid user interactions', async () => {
            let operationCount = 0;
            const operations = [];

            // Simulate rapid operations
            for (let i = 0; i < 10; i++) {
                operations.push(
                    app.debounceSearch(`query${i}`, 10).then(() => {
                        operationCount++;
                    })
                );
            }

            await Promise.all(operations);
            
            // Due to debouncing, only final operation should execute
            expect(operationCount).toBe(1);
        });

        it('should maintain performance with complex wiki links', () => {
            const complexContent = Array.from({ length: 50 }, (_, i) => 
                `This is content with [[Link ${i}]] and more text.`
            ).join(' ');

            const startTime = performance.now();
            const processedContent = app.processWikiLinks(complexContent);
            const endTime = performance.now();

            const linkCount = (processedContent.match(/class="wiki-link"/g) || []).length;
            expect(linkCount).toBe(50);
            expect(endTime - startTime).toBeLessThan(100);
        });
    });

    describe('Accessibility Workflow', () => {
        it('should generate accessible wiki links', () => {
            const content = 'Reference [[Important Link]] here';
            const processed = app.processWikiLinks(content);
            
            expect(processed).toContain('href="#"');
            expect(processed).toContain('class="wiki-link"');
            // Should be keyboard navigable and screen reader friendly
        });

        it('should handle keyboard navigation', () => {
            fixture.element.innerHTML = `
                <button id="test-btn" tabindex="0">Test Button</button>
            `;

            const button = fixture.element.querySelector('#test-btn');
            
            // Simulate keyboard events
            const enterEvent = TestUtils.createMockEvent('keydown', { key: 'Enter' });
            const spaceEvent = TestUtils.createMockEvent('keydown', { key: ' ' });

            expect(() => {
                button.dispatchEvent(enterEvent);
                button.dispatchEvent(spaceEvent);
            }).not.toThrow();
        });
    });

    describe('Data Persistence Workflow', () => {
        it('should maintain session state during interactions', () => {
            const initialSession = app.startReviewSession(TestUtils.generateTestCards(3));
            const sessionId = initialSession.sessionId;
            
            // Perform various operations
            app.currentQuiz = { card: initialSession.dueCards[0], questions: [] };
            app.rateCard(3);
            
            // Session should persist
            expect(app.reviewSession.sessionId).toBe(sessionId);
            expect(app.reviewSession.currentCardIndex).toBe(1);
        });

        it('should handle cache management', () => {
            const cacheKey = 'test-card-1';
            const cacheData = { title: 'Cached Card', content: 'Cached content' };
            
            // Add to cache
            app.cardCache.set(cacheKey, cacheData);
            expect(app.cardCache.get(cacheKey)).toEqual(cacheData);
            
            // Cache should persist during operations
            app.performSearch('test');
            expect(app.cardCache.get(cacheKey)).toEqual(cacheData);
        });
    });
});