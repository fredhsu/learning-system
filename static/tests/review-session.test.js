/**
 * Unit Tests for Review Session Functionality
 */

describe('Review Session Management', () => {
    let app;
    let testCards;
    
    beforeEach(() => {
        app = new LearningSystemTest();
        testCards = [
            { id: 1, title: 'Card 1', content: 'Content 1', zettel_id: '1.1' },
            { id: 2, title: 'Card 2', content: 'Content 2', zettel_id: '1.2' },
            { id: 3, title: 'Card 3', content: 'Content 3', zettel_id: '1.3' }
        ];
    });

    describe('Session Initialization', () => {
        it('should initialize review session with cards', () => {
            const session = app.startReviewSession(testCards);
            
            expect(session.totalCards).toBe(3);
            expect(session.currentCardIndex).toBe(0);
            expect(session.dueCards).toEqual(testCards);
            expect(session.sessionId).toMatch(/^session-\d+$/);
            expect(session.startTime).toBeInstanceOf(Date);
        });

        it('should initialize empty review session', () => {
            const session = app.startReviewSession([]);
            
            expect(session.totalCards).toBe(0);
            expect(session.dueCards).toEqual([]);
        });

        it('should reset session counters', () => {
            // Set some initial values
            app.reviewSession.totalQuestions = 10;
            app.reviewSession.correctAnswers = 5;
            app.reviewSession.currentCardIndex = 2;
            
            const session = app.startReviewSession(testCards);
            
            expect(session.totalQuestions).toBe(0);
            expect(session.correctAnswers).toBe(0);
            expect(session.currentCardIndex).toBe(0);
        });

        it('should generate unique session IDs', () => {
            const session1 = app.startReviewSession(testCards);
            const session2 = app.startReviewSession(testCards);
            
            expect(session1.sessionId).not.toBe(session2.sessionId);
        });
    });

    describe('Card Rating', () => {
        beforeEach(() => {
            app.startReviewSession(testCards);
            app.currentQuiz = { card: testCards[0], questions: [] };
        });

        it('should rate card and advance session', () => {
            const hasMoreCards = app.rateCard(3);
            
            expect(app.reviewSession.currentCardIndex).toBe(1);
            expect(hasMoreCards).toBe(true);
        });

        it('should complete session when rating last card', () => {
            // Move to last card
            app.reviewSession.currentCardIndex = 2;
            
            const hasMoreCards = app.rateCard(4);
            
            expect(app.reviewSession.currentCardIndex).toBe(3);
            expect(hasMoreCards).toBe(false);
        });

        it('should handle rating without current quiz', () => {
            app.currentQuiz = null;
            
            const result = app.rateCard(3);
            
            expect(result).toBe(false);
        });

        it('should handle different rating values', () => {
            const ratings = [1, 2, 3, 4];
            
            ratings.forEach(rating => {
                app.reviewSession.currentCardIndex = 0;
                app.currentQuiz = { card: testCards[0], questions: [] };
                
                expect(() => app.rateCard(rating)).not.toThrow();
            });
        });
    });

    describe('Rating Calculation', () => {
        it('should calculate average rating correctly', () => {
            const ratings = [2, 3, 4];
            const result = app.calculateFinalRating(ratings);
            
            expect(result).toBe(3); // Math.round(9/3) = 3
        });

        it('should handle single rating', () => {
            const ratings = [4];
            const result = app.calculateFinalRating(ratings);
            
            expect(result).toBe(4);
        });

        it('should round ratings properly', () => {
            // Test rounding down
            const ratings1 = [2, 2, 3]; // Average: 2.33 -> rounds to 2
            expect(app.calculateFinalRating(ratings1)).toBe(2);
            
            // Test rounding up
            const ratings2 = [3, 3, 4]; // Average: 3.33 -> rounds to 3
            expect(app.calculateFinalRating(ratings2)).toBe(3);
            
            // Test rounding at 0.5
            const ratings3 = [2, 3]; // Average: 2.5 -> rounds to 3
            expect(app.calculateFinalRating(ratings3)).toBe(3);
        });

        it('should handle empty ratings array', () => {
            const result = app.calculateFinalRating([]);
            
            expect(result).toBe(3); // Default to "Good"
        });

        it('should handle null/undefined ratings', () => {
            expect(app.calculateFinalRating(null)).toBe(3);
            expect(app.calculateFinalRating(undefined)).toBe(3);
        });

        it('should handle invalid ratings', () => {
            const ratings = ['invalid', null, 3, undefined, 2];
            const result = app.calculateFinalRating(ratings);
            
            // Should ignore invalid values and calculate from valid ones
            expect(result).toBe(3); // (3 + 2) / 2 = 2.5 -> rounds to 3
        });

        it('should handle extreme rating values', () => {
            const ratings = [1, 1, 1, 4, 4, 4];
            const result = app.calculateFinalRating(ratings);
            
            expect(result).toBe(3); // (1+1+1+4+4+4) / 6 = 2.5 -> rounds to 3
        });
    });

    describe('Session Progress Tracking', () => {
        beforeEach(() => {
            app.startReviewSession(testCards);
        });

        it('should track session progress correctly', () => {
            expect(app.reviewSession.currentCardIndex).toBe(0);
            expect(app.reviewSession.totalCards).toBe(3);
            
            // Progress: 0/3 = 0%
            const progress1 = (app.reviewSession.currentCardIndex / app.reviewSession.totalCards) * 100;
            expect(progress1).toBe(0);
            
            // Advance session
            app.currentQuiz = { card: testCards[0], questions: [] };
            app.rateCard(3);
            
            // Progress: 1/3 = 33.33%
            const progress2 = (app.reviewSession.currentCardIndex / app.reviewSession.totalCards) * 100;
            expect(Math.round(progress2)).toBe(33);
        });

        it('should calculate remaining cards correctly', () => {
            expect(app.reviewSession.totalCards - app.reviewSession.currentCardIndex).toBe(3);
            
            app.currentQuiz = { card: testCards[0], questions: [] };
            app.rateCard(3);
            
            expect(app.reviewSession.totalCards - app.reviewSession.currentCardIndex).toBe(2);
        });

        it('should handle session completion', () => {
            // Complete all cards
            app.currentQuiz = { card: testCards[0], questions: [] };
            app.rateCard(3);
            app.currentQuiz = { card: testCards[1], questions: [] };
            app.rateCard(3);
            app.currentQuiz = { card: testCards[2], questions: [] };
            app.rateCard(3);
            
            const remainingCards = app.reviewSession.totalCards - app.reviewSession.currentCardIndex;
            expect(remainingCards).toBe(0);
        });
    });

    describe('Session State Management', () => {
        it('should maintain session state across operations', () => {
            const session = app.startReviewSession(testCards);
            const originalSessionId = session.sessionId;
            const originalStartTime = session.startTime;
            
            // Perform some operations
            app.currentQuiz = { card: testCards[0], questions: [] };
            app.rateCard(3);
            
            // Session state should be maintained
            expect(app.reviewSession.sessionId).toBe(originalSessionId);
            expect(app.reviewSession.startTime).toBe(originalStartTime);
            expect(app.reviewSession.totalCards).toBe(3);
        });

        it('should reset session when starting new review', () => {
            const newCards = [testCards[0]];
            app.startReviewSession(testCards);
            const session1Id = app.reviewSession.sessionId;
            
            app.startReviewSession(newCards);
            const session2Id = app.reviewSession.sessionId;
            
            expect(session2Id).not.toBe(session1Id);
            expect(app.reviewSession.totalCards).toBe(1);
            expect(app.reviewSession.currentCardIndex).toBe(0);
        });

        it('should handle concurrent session access', () => {
            app.startReviewSession(testCards);
            const session = app.reviewSession;
            
            // Simulate concurrent access
            const sessionCopy1 = { ...session };
            const sessionCopy2 = { ...session };
            
            expect(sessionCopy1.sessionId).toBe(sessionCopy2.sessionId);
            expect(sessionCopy1.totalCards).toBe(sessionCopy2.totalCards);
        });
    });

    describe('Edge Cases and Error Handling', () => {
        it('should handle session with single card', () => {
            const singleCard = [testCards[0]];
            app.startReviewSession(singleCard);
            
            expect(app.reviewSession.totalCards).toBe(1);
            
            app.currentQuiz = { card: singleCard[0], questions: [] };
            const hasMore = app.rateCard(3);
            
            expect(hasMore).toBe(false);
            expect(app.reviewSession.currentCardIndex).toBe(1);
        });

        it('should handle malformed card data', () => {
            const malformedCards = [
                { id: 1 }, // Missing required fields
                { title: 'Title Only' }, // Missing id
                null, // Null card
                undefined // Undefined card
            ];
            
            expect(() => app.startReviewSession(malformedCards)).not.toThrow();
            expect(app.reviewSession.totalCards).toBe(4);
        });

        it('should handle very large card sets', () => {
            const largeCardSet = Array.from({ length: 1000 }, (_, i) => ({
                id: i,
                title: `Card ${i}`,
                content: `Content ${i}`,
                zettel_id: `${i}.1`
            }));
            
            const startTime = performance.now();
            app.startReviewSession(largeCardSet);
            const endTime = performance.now();
            
            expect(app.reviewSession.totalCards).toBe(1000);
            expect(endTime - startTime).toBeLessThan(100); // Should be fast
        });

        it('should maintain session integrity after errors', () => {
            app.startReviewSession(testCards);
            const originalSessionId = app.reviewSession.sessionId;
            
            // Simulate error condition
            app.currentQuiz = null;
            app.rateCard(3); // This should fail gracefully
            
            // Session should still be intact
            expect(app.reviewSession.sessionId).toBe(originalSessionId);
            expect(app.reviewSession.totalCards).toBe(3);
        });
    });
});