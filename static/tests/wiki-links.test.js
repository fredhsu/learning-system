/**
 * Unit Tests for Wiki Link Processing
 */

describe('Wiki Link Processing', () => {
    let app;
    
    beforeEach(() => {
        app = new LearningSystemTest();
    });

    describe('Link Pattern Recognition', () => {
        it('should recognize simple wiki links', () => {
            const testCases = [
                { input: '[[Simple Link]]', expected: 'Simple Link' },
                { input: '[[Another Link]]', expected: 'Another Link' },
                { input: '[[123]]', expected: '123' }
            ];

            testCases.forEach(({ input, expected }) => {
                const result = app.processWikiLinks(input);
                expect(result).toContain(`data-link-text="${expected}"`);
                expect(result).toContain(`<span>${expected}</span>`);
            });
        });

        it('should handle multiple links in same text', () => {
            const input = 'See [[Link 1]] and [[Link 2]] for details';
            const result = app.processWikiLinks(input);
            
            expect(result).toContain('data-link-text="Link 1"');
            expect(result).toContain('data-link-text="Link 2"');
            expect(result).toContain('for details');
        });

        it('should preserve text around links', () => {
            const input = 'Before [[Link]] after text';
            const result = app.processWikiLinks(input);
            
            expect(result).toContain('Before');
            expect(result).toContain('after text');
            expect(result).toContain('class="wiki-link"');
        });

        it('should handle links at text boundaries', () => {
            const testCases = [
                '[[Start Link]] at beginning',
                'At end [[End Link]]',
                '[[Only Link]]'
            ];

            testCases.forEach(input => {
                const result = app.processWikiLinks(input);
                expect(result).toContain('class="wiki-link"');
            });
        });
    });

    describe('Link Content Processing', () => {
        it('should handle links with spaces', () => {
            const input = '[[Multiple Word Link]]';
            const result = app.processWikiLinks(input);
            
            expect(result).toContain('data-link-text="Multiple Word Link"');
            expect(result).toContain('<span>Multiple Word Link</span>');
        });

        it('should handle links with numbers', () => {
            const input = '[[Chapter 1.2.3]]';
            const result = app.processWikiLinks(input);
            
            expect(result).toContain('data-link-text="Chapter 1.2.3"');
        });

        it('should handle links with special characters', () => {
            const testCases = [
                '[[Link-With-Dashes]]',
                '[[Link_With_Underscores]]',
                '[[Link (with parentheses)]]',
                '[[Link & Symbols!]]'
            ];

            testCases.forEach(input => {
                const result = app.processWikiLinks(input);
                expect(result).toContain('class="wiki-link"');
            });
        });

        it('should trim whitespace from link text', () => {
            const testCases = [
                { input: '[[  Spaced Link  ]]', expected: 'Spaced Link' },
                { input: '[[ \t Tabbed Link \t ]]', expected: 'Tabbed Link' },
                { input: '[[\n\nNewline Link\n\n]]', expected: 'Newline Link' }
            ];

            testCases.forEach(({ input, expected }) => {
                const result = app.processWikiLinks(input);
                expect(result).toContain(`data-link-text="${expected}"`);
                expect(result).toContain(`<span>${expected}</span>`);
            });
        });

        it('should handle empty links', () => {
            const input = '[[]]';
            const result = app.processWikiLinks(input);
            
            expect(result).toContain('data-link-text=""');
            expect(result).toContain('<span></span>');
        });

        it('should handle links with only whitespace', () => {
            const input = '[[   ]]';
            const result = app.processWikiLinks(input);
            
            expect(result).toContain('data-link-text=""');
            expect(result).toContain('<span></span>');
        });
    });

    describe('HTML Generation', () => {
        it('should generate proper HTML structure', () => {
            const input = '[[Test Link]]';
            const result = app.processWikiLinks(input);
            
            expect(result).toContain('<a href="#"');
            expect(result).toContain('class="wiki-link"');
            expect(result).toContain('data-link-text="Test Link"');
            expect(result).toContain('<i data-feather="link" class="wiki-link-icon"></i>');
            expect(result).toContain('<span>Test Link</span>');
            expect(result).toContain('</a>');
        });

        it('should generate unique IDs for each link', () => {
            const input = '[[Link 1]] and [[Link 2]]';
            const result = app.processWikiLinks(input);
            
            const idMatches = result.match(/id="[^"]+"/g);
            expect(idMatches).toHaveLength(2);
            expect(idMatches[0]).not.toBe(idMatches[1]);
        });

        it('should include feather icon markup', () => {
            const input = '[[Test Link]]';
            const result = app.processWikiLinks(input);
            
            expect(result).toContain('data-feather="link"');
            expect(result).toContain('class="wiki-link-icon"');
        });

        it('should preserve link text in span', () => {
            const testCases = [
                'Simple Link',
                'Complex Link With Many Words',
                'Link123',
                'Link-with-dashes'
            ];

            testCases.forEach(linkText => {
                const input = `[[${linkText}]]`;
                const result = app.processWikiLinks(input);
                expect(result).toContain(`<span>${linkText}</span>`);
            });
        });
    });

    describe('Edge Cases', () => {
        it('should handle nested brackets', () => {
            const input = '[[Link with [nested] brackets]]';
            const result = app.processWikiLinks(input);
            
            expect(result).toContain('data-link-text="Link with [nested] brackets"');
        });

        it('should handle malformed brackets', () => {
            const testCases = [
                '[Single bracket]',
                '[[Unclosed link',
                'Unopened link]]',
                '[[[Triple brackets]]]'
            ];

            testCases.forEach(input => {
                expect(() => app.processWikiLinks(input)).not.toThrow();
            });
        });

        it('should handle very long link text', () => {
            const longLinkText = 'A'.repeat(1000);
            const input = `[[${longLinkText}]]`;
            const result = app.processWikiLinks(input);
            
            expect(result).toContain(`data-link-text="${longLinkText}"`);
            expect(result).toContain(`<span>${longLinkText}</span>`);
        });

        it('should handle unicode characters', () => {
            const testCases = [
                '[[Café]]',
                '[[Naïve]]',
                '[[Résumé]]',
                '[[🔗 Link]]',
                '[[中文链接]]'
            ];

            testCases.forEach(input => {
                const result = app.processWikiLinks(input);
                expect(result).toContain('class="wiki-link"');
            });
        });

        it('should handle text with many links', () => {
            const links = Array.from({ length: 100 }, (_, i) => `[[Link ${i}]]`);
            const input = links.join(' ');
            
            const startTime = performance.now();
            const result = app.processWikiLinks(input);
            const endTime = performance.now();
            
            const linkCount = (result.match(/class="wiki-link"/g) || []).length;
            expect(linkCount).toBe(100);
            expect(endTime - startTime).toBeLessThan(100); // Should be fast
        });
    });

    describe('Integration with Markdown', () => {
        it('should work with markdown processing', () => {
            const input = '**Bold** text with [[Wiki Link]] and *italic*';
            const result = app.renderMarkdown(input);
            
            expect(result).toContain('<strong>Bold</strong>');
            expect(result).toContain('class="wiki-link"');
            expect(result).toContain('<em>italic</em>');
        });

        it('should process wiki links before markdown', () => {
            const input = '[[**Bold Link**]]';
            const result = app.renderMarkdown(input);
            
            // Link should be processed first, so markdown inside stays as text
            expect(result).toContain('data-link-text="**Bold Link**"');
            expect(result).toContain('<span>**Bold Link**</span>');
        });

        it('should handle mixed content correctly', () => {
            const input = 'See [[Link 1]] and **bold** and [[Link 2]]';
            const result = app.renderMarkdown(input);
            
            expect(result).toContain('data-link-text="Link 1"');
            expect(result).toContain('data-link-text="Link 2"');
            expect(result).toContain('<strong>bold</strong>');
        });
    });

    describe('Preview Content Generation', () => {
        it('should include wiki links in preview', () => {
            const input = 'Short content with [[Link]]';
            const result = app.createPreviewContent(input);
            
            expect(result).toContain('class="wiki-link"');
            expect(result).toContain('data-link-text="Link"');
        });

        it('should truncate content but preserve complete links', () => {
            const longContent = 'A'.repeat(50) + '[[Important Link]]' + 'B'.repeat(50);
            const result = app.createPreviewContent(longContent);
            
            // Should be truncated to 100 characters but links should be processed
            expect(result.length).toBeLessThanOrEqual(150); // Account for link HTML
            expect(result).toContain('class="wiki-link"');
        });

        it('should handle preview with multiple links', () => {
            const input = 'Content with [[Link 1]] and [[Link 2]]';
            const result = app.createPreviewContent(input);
            
            expect(result).toContain('data-link-text="Link 1"');
            expect(result).toContain('data-link-text="Link 2"');
        });
    });
});