/**
 * Unit Tests for Search Functionality
 */

describe('Search Functionality', () => {
    let app;
    let testCards;
    
    beforeEach(() => {
        app = new LearningSystemTest();
        testCards = [
            { id: 1, title: 'Mathematics Basics', content: 'Algebra and geometry fundamentals', zettel_id: '1.1' },
            { id: 2, title: 'Physics Concepts', content: 'Newton laws and thermodynamics', zettel_id: '1.2' },
            { id: 3, title: 'Chemistry Notes', content: 'Periodic table and chemical reactions', zettel_id: '1.3' },
            { id: 4, title: 'Advanced Mathematics', content: 'Calculus and linear algebra concepts', zettel_id: '2.1' }
        ];
        app.allCards = testCards;
    });

    describe('Search Execution', () => {
        it('should return all cards for empty query', () => {
            const results = app.performSearch('');
            expect(results).toEqual(testCards);
        });

        it('should return all cards for null query', () => {
            const results = app.performSearch(null);
            expect(results).toEqual(testCards);
        });

        it('should search by title (case insensitive)', () => {
            const results = app.performSearch('mathematics');
            
            expect(results).toHaveLength(2);
            expect(results.some(card => card.title.includes('Mathematics'))).toBe(true);
            expect(results.some(card => card.title.includes('Advanced Mathematics'))).toBe(true);
        });

        it('should search by content', () => {
            const results = app.performSearch('algebra');
            
            expect(results).toHaveLength(2);
            expect(results.some(card => card.content.includes('Algebra'))).toBe(true);
            expect(results.some(card => card.content.includes('linear algebra'))).toBe(true);
        });

        it('should handle partial matches', () => {
            const results = app.performSearch('math');
            
            expect(results).toHaveLength(2);
        });

        it('should return empty array for no matches', () => {
            const results = app.performSearch('nonexistent');
            
            expect(results).toHaveLength(0);
        });

        it('should handle special characters in search', () => {
            app.allCards.push({ 
                id: 5, 
                title: 'Test & Special', 
                content: 'Contains & and other symbols!', 
                zettel_id: '3.1' 
            });
            
            const results = app.performSearch('&');
            expect(results).toHaveLength(1);
        });
    });

    describe('Search Highlighting', () => {
        it('should highlight single search terms', () => {
            const text = 'This is a test string';
            const result = app.highlightSearchTerms(text, 'test');
            
            expect(result).toBe('This is a <span class="search-highlight">test</span> string');
        });

        it('should highlight case insensitive matches', () => {
            const text = 'Mathematics and MATHEMATICS';
            const result = app.highlightSearchTerms(text, 'mathematics');
            
            expect(result).toContain('<span class="search-highlight">Mathematics</span>');
            expect(result).toContain('<span class="search-highlight">MATHEMATICS</span>');
        });

        it('should return original text for empty query', () => {
            const text = 'Original text';
            const result = app.highlightSearchTerms(text, '');
            
            expect(result).toBe(text);
        });

        it('should return original text for null query', () => {
            const text = 'Original text';
            const result = app.highlightSearchTerms(text, null);
            
            expect(result).toBe(text);
        });

        it('should highlight multiple occurrences', () => {
            const text = 'test test test';
            const result = app.highlightSearchTerms(text, 'test');
            
            const highlightCount = (result.match(/search-highlight/g) || []).length;
            expect(highlightCount).toBe(3);
        });

        it('should handle special regex characters', () => {
            const text = 'Math (basics) and [advanced]';
            const result = app.highlightSearchTerms(text, '(basics)');
            
            expect(result).toContain('<span class="search-highlight">(basics)</span>');
        });
    });

    describe('Search Debouncing', () => {
        it('should debounce search calls', async () => {
            let searchCount = 0;
            const originalPerformSearch = app.performSearch;
            app.performSearch = () => {
                searchCount++;
                return originalPerformSearch.call(app, 'test');
            };

            // Trigger multiple rapid searches
            app.debounceSearch('test1', 50);
            app.debounceSearch('test2', 50);
            app.debounceSearch('test3', 50);
            
            // Wait for debounce
            await TestUtils.wait(100);
            
            expect(searchCount).toBe(1);
        });

        it('should execute search after debounce delay', async () => {
            const promise = app.debounceSearch('mathematics', 50);
            const results = await promise;
            
            expect(Array.isArray(results)).toBe(true);
            expect(results.length).toBeGreaterThan(0);
        });

        it('should cancel previous search when new one starts', async () => {
            const search1 = app.debounceSearch('math', 100);
            const search2 = app.debounceSearch('physics', 50);
            
            const results = await search2;
            expect(results.some(card => card.title.includes('Physics'))).toBe(true);
        });
    });

    describe('Search Performance', () => {
        beforeEach(() => {
            // Create a large dataset
            const largeDataset = [];
            for (let i = 0; i < 1000; i++) {
                largeDataset.push({
                    id: i,
                    title: `Card ${i}`,
                    content: `Content for card ${i} with various keywords`,
                    zettel_id: `${Math.floor(i / 100)}.${i % 100}`
                });
            }
            app.allCards = largeDataset;
        });

        it('should handle large datasets efficiently', () => {
            const startTime = performance.now();
            const results = app.performSearch('card');
            const endTime = performance.now();
            
            expect(results.length).toBeGreaterThan(0);
            expect(endTime - startTime).toBeLessThan(100); // Should complete within 100ms
        });

        it('should maintain performance with complex queries', () => {
            const startTime = performance.now();
            const results = app.performSearch('keywords');
            const endTime = performance.now();
            
            expect(results.length).toBeGreaterThan(0);
            expect(endTime - startTime).toBeLessThan(100);
        });
    });

    describe('Search Edge Cases', () => {
        it('should handle undefined cards array', () => {
            app.allCards = undefined;
            
            expect(() => app.performSearch('test')).toThrow();
        });

        it('should handle empty cards array', () => {
            app.allCards = [];
            const results = app.performSearch('test');
            
            expect(results).toEqual([]);
        });

        it('should handle cards with missing properties', () => {
            app.allCards = [
                { id: 1 }, // Missing title and content
                { id: 2, title: 'Title Only' }, // Missing content
                { id: 3, content: 'Content Only' } // Missing title
            ];
            
            expect(() => app.performSearch('test')).not.toThrow();
            const results = app.performSearch('Title');
            expect(results).toHaveLength(1);
        });

        it('should handle very long search queries', () => {
            const longQuery = 'a'.repeat(1000);
            const results = app.performSearch(longQuery);
            
            expect(Array.isArray(results)).toBe(true);
        });

        it('should handle search with only whitespace', () => {
            const results = app.performSearch('   ');
            
            expect(results).toEqual(testCards);
        });

        it('should handle unicode characters in search', () => {
            app.allCards.push({
                id: 5,
                title: 'Unicode Test: café, naïve, résumé',
                content: 'Content with émojis 🔍 and açcénts',
                zettel_id: '4.1'
            });
            
            const results = app.performSearch('café');
            expect(results).toHaveLength(1);
        });
    });
});