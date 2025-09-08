/**
 * Unit Tests for LearningSystem Core Functionality
 */

describe('LearningSystem Core', () => {
    let app;
    
    beforeEach(() => {
        app = new LearningSystemTest();
    });

    describe('Initialization', () => {
        it('should initialize with default values', () => {
            expect(app.baseURL).toBe('/api');
            expect(app.currentView).toBe('cards');
            expect(app.allCards).toEqual([]);
            expect(app.reviewSession.totalCards).toBe(0);
        });

        it('should initialize empty card cache', () => {
            expect(app.cardCache).toBeInstanceOf(Map);
            expect(app.cardCache.size).toBe(0);
        });

        it('should initialize preview timeouts map', () => {
            expect(app.previewTimeouts).toBeInstanceOf(Map);
            expect(app.previewTimeouts.size).toBe(0);
        });
    });

    describe('Wiki Link Processing', () => {
        it('should process simple wiki links', () => {
            const content = 'This is a [[Simple Link]] in text';
            const result = app.processWikiLinks(content);
            
            expect(result).toContain('class="wiki-link"');
            expect(result).toContain('data-link-text="Simple Link"');
            expect(result).toContain('<span>Simple Link</span>');
        });

        it('should process multiple wiki links', () => {
            const content = 'Links: [[Link 1]] and [[Link 2]]';
            const result = app.processWikiLinks(content);
            
            expect(result).toContain('data-link-text="Link 1"');
            expect(result).toContain('data-link-text="Link 2"');
        });

        it('should handle wiki links with special characters', () => {
            const content = 'Link: [[Complex Link-Name_123]]';
            const result = app.processWikiLinks(content);
            
            expect(result).toContain('data-link-text="Complex Link-Name_123"');
        });

        it('should handle content without wiki links', () => {
            const content = 'This is plain text without links';
            const result = app.processWikiLinks(content);
            
            expect(result).toBe(content);
        });

        it('should trim whitespace in wiki links', () => {
            const content = 'Link: [[  Spaced Link  ]]';
            const result = app.processWikiLinks(content);
            
            expect(result).toContain('data-link-text="Spaced Link"');
            expect(result).toContain('<span>Spaced Link</span>');
        });
    });

    describe('Markdown Rendering', () => {
        it('should render bold text', () => {
            const content = 'This is **bold** text';
            const result = app.renderMarkdown(content);
            
            expect(result).toContain('<strong>bold</strong>');
        });

        it('should render italic text', () => {
            const content = 'This is *italic* text';
            const result = app.renderMarkdown(content);
            
            expect(result).toContain('<em>italic</em>');
        });

        it('should render inline code', () => {
            const content = 'This is `code` text';
            const result = app.renderMarkdown(content);
            
            expect(result).toContain('<code>code</code>');
        });

        it('should convert newlines to break tags', () => {
            const content = 'Line 1\nLine 2';
            const result = app.renderMarkdown(content);
            
            expect(result).toContain('Line 1<br>Line 2');
        });

        it('should process wiki links before markdown', () => {
            const content = '**Bold** and [[Wiki Link]]';
            const result = app.renderMarkdown(content);
            
            expect(result).toContain('<strong>Bold</strong>');
            expect(result).toContain('class="wiki-link"');
        });
    });

    describe('Preview Content Creation', () => {
        it('should create preview with character limit', () => {
            const longContent = 'a'.repeat(200);
            const result = app.createPreviewContent(longContent);
            
            expect(result.length).toBeLessThanOrEqual(100);
        });

        it('should process wiki links in preview', () => {
            const content = 'Short [[Link]] content';
            const result = app.createPreviewContent(content);
            
            expect(result).toContain('class="wiki-link"');
        });

        it('should remove markdown characters', () => {
            const content = '**Bold** *italic* `code`';
            const result = app.createPreviewContent(content);
            
            expect(result).toBe('Bold italic code');
        });
    });
});

describe('LearningSystem API', () => {
    let app;
    
    beforeEach(() => {
        app = new LearningSystemTest();
    });

    describe('API Calls', () => {
        it('should make successful API call', async () => {
            const result = await app.apiCall('/cards');
            
            expect(result.success).toBe(true);
            expect(result.data).toBeDefined();
        });

        it('should handle API timeout', async () => {
            const startTime = Date.now();
            await app.apiCall('/cards');
            const endTime = Date.now();
            
            expect(endTime - startTime).toBeGreaterThanOrEqual(10);
        });

        it('should return mock data for cards endpoint', async () => {
            const result = await app.apiCall('/cards');
            
            expect(Array.isArray(result.data)).toBe(true);
            expect(result.data.length).toBeGreaterThan(0);
            expect(result.data[0]).toHaveProperty('id');
            expect(result.data[0]).toHaveProperty('zettel_id');
        });
    });
});

describe('LearningSystem Validation', () => {
    let app;
    
    beforeEach(() => {
        app = new LearningSystemTest();
    });

    describe('Zettel ID Validation', () => {
        it('should validate correct Zettel IDs', () => {
            const validIds = ['1', '1.1', '1.1.1', '1a', '1.1a', '10.20.30'];
            
            validIds.forEach(id => {
                const result = app.validateZettelId(id);
                expect(result.valid).toBe(true);
            });
        });

        it('should reject invalid Zettel IDs', () => {
            const invalidIds = ['', 'abc', '1..1', '.1', '1.', '1-1', '1 1'];
            
            invalidIds.forEach(id => {
                const result = app.validateZettelId(id);
                expect(result.valid).toBe(false);
                expect(result.error).toBeDefined();
            });
        });

        it('should handle null and undefined IDs', () => {
            expect(app.validateZettelId(null).valid).toBe(false);
            expect(app.validateZettelId(undefined).valid).toBe(false);
        });

        it('should handle non-string IDs', () => {
            expect(app.validateZettelId(123).valid).toBe(false);
            expect(app.validateZettelId({}).valid).toBe(false);
        });
    });

    describe('Input Sanitization', () => {
        it('should sanitize HTML characters', () => {
            const input = '<script>alert("xss")</script>';
            const result = app.sanitizeInput(input);
            
            expect(result).toBe('&lt;script&gt;alert(&quot;xss&quot;)&lt;&#x2F;script&gt;');
        });

        it('should handle quotes and apostrophes', () => {
            const input = 'It\'s a "test"';
            const result = app.sanitizeInput(input);
            
            expect(result).toBe('It&#x27;s a &quot;test&quot;');
        });

        it('should handle non-string input', () => {
            expect(app.sanitizeInput(null)).toBe('');
            expect(app.sanitizeInput(undefined)).toBe('');
            expect(app.sanitizeInput(123)).toBe('');
        });

        it('should preserve safe characters', () => {
            const input = 'Safe text with numbers 123 and symbols !@#$%^&*()';
            const result = app.sanitizeInput(input);
            
            expect(result).toContain('Safe text');
            expect(result).toContain('123');
            expect(result).toContain('!@#$%^&*()');
        });
    });
});

describe('LearningSystem Error Handling', () => {
    let app;
    
    beforeEach(() => {
        app = new LearningSystemTest();
    });

    describe('Error Messages', () => {
        it('should show error messages', () => {
            const result = app.showError('Test error message');
            
            expect(result.type).toBe('error');
            expect(result.message).toBe('Test error message');
        });

        it('should show success messages', () => {
            const result = app.showSuccess('Test success message');
            
            expect(result.type).toBe('success');
            expect(result.message).toBe('Test success message');
        });
    });

    describe('Modal Management', () => {
        it('should show modal', () => {
            const result = app.showModal('test-modal');
            
            expect(result.modalId).toBe('test-modal');
            expect(result.visible).toBe(true);
        });

        it('should close modal', () => {
            const modal = { id: 'test-modal' };
            const result = app.closeModal(modal);
            
            expect(result.modal).toBe(modal);
            expect(result.visible).toBe(false);
        });
    });
});