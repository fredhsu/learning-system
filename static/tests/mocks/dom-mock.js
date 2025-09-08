/**
 * DOM Mock for testing without full DOM environment
 */

// Create a simplified LearningSystem class for testing
class LearningSystemTest {
    constructor() {
        this.baseURL = '/api';
        this.currentView = 'cards';
        this.currentQuiz = null;
        this.searchDebounceTimer = null;
        this.currentSearchQuery = '';
        this.allCards = [];
        this.reviewSession = {
            totalCards: 0,
            currentCardIndex: 0,
            totalQuestions: 0,
            correctAnswers: 0,
            startTime: null,
            dueCards: [],
            sessionId: null,
            questions: {}
        };
        this.cardCache = new Map();
        this.previewTimeouts = new Map();
    }

    // Core methods from the original class
    processWikiLinks(content) {
        return content.replace(/\[\[([^\]]+)\]\]/g, (match, linkText) => {
            const trimmedText = linkText.trim();
            const uniqueId = `link-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
            
            return `<a href="#" class="wiki-link" id="${uniqueId}" 
                       data-link-text="${trimmedText}">
                <i data-feather="link" class="wiki-link-icon"></i>
                <span>${trimmedText}</span>
            </a>`;
        });
    }

    renderMarkdown(content) {
        const processedContent = this.processWikiLinks(content);
        // Simplified markdown processing
        return processedContent
            .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
            .replace(/\*(.*?)\*/g, '<em>$1</em>')
            .replace(/`(.*?)`/g, '<code>$1</code>')
            .replace(/\n/g, '<br>');
    }

    createPreviewContent(content) {
        const linkedContent = this.processWikiLinks(content);
        const plainTextContent = linkedContent.replace(/[#*_`~]/g, '').substring(0, 100);
        return plainTextContent.replace(/\n/g, '<br>');
    }

    // API mock method
    async apiCall(endpoint, options = {}) {
        // Mock API responses for testing
        const mockResponses = {
            '/cards': [
                { id: 1, zettel_id: '1.1', title: 'Test Card 1', content: 'Content 1' },
                { id: 2, zettel_id: '1.2', title: 'Test Card 2', content: 'Content 2' }
            ],
            '/topics': [
                { id: 1, name: 'Mathematics', description: 'Math topics' }
            ]
        };

        return new Promise((resolve) => {
            setTimeout(() => {
                resolve({ success: true, data: mockResponses[endpoint] || [] });
            }, 10);
        });
    }

    // Search functionality
    debounceSearch(query, delay = 300) {
        return new Promise((resolve) => {
            clearTimeout(this.searchDebounceTimer);
            this.searchDebounceTimer = setTimeout(() => {
                resolve(this.performSearch(query));
            }, delay);
        });
    }

    performSearch(query) {
        if (!query) return this.allCards;
        
        return this.allCards.filter(card => 
            card.title.toLowerCase().includes(query.toLowerCase()) ||
            card.content.toLowerCase().includes(query.toLowerCase())
        );
    }

    highlightSearchTerms(text, query) {
        if (!query) return text;
        
        const regex = new RegExp(`(${query})`, 'gi');
        return text.replace(regex, '<span class="search-highlight">$1</span>');
    }

    // Review session methods
    startReviewSession(cards = []) {
        this.reviewSession = {
            totalCards: cards.length,
            currentCardIndex: 0,
            totalQuestions: 0,
            correctAnswers: 0,
            startTime: new Date(),
            dueCards: cards,
            sessionId: `session-${Date.now()}`,
            questions: {}
        };
        return this.reviewSession;
    }

    rateCard(rating) {
        if (!this.currentQuiz) return false;
        
        // Mock rating logic
        this.reviewSession.currentCardIndex++;
        return this.reviewSession.currentCardIndex < this.reviewSession.totalCards;
    }

    calculateFinalRating(questionRatings) {
        if (!Array.isArray(questionRatings) || questionRatings.length === 0) {
            return 3; // Default to "Good"
        }
        
        const average = questionRatings.reduce((sum, rating) => sum + rating, 0) / questionRatings.length;
        return Math.round(average);
    }

    // Validation methods
    validateZettelId(zettelId) {
        if (!zettelId || typeof zettelId !== 'string') {
            return { valid: false, error: 'Zettel ID is required' };
        }
        
        // Validate format: numbers and dots, optionally with letters
        const zettelPattern = /^[\d]+(?:\.[\d]+)*[a-z]*$/;
        if (!zettelPattern.test(zettelId)) {
            return { valid: false, error: 'Invalid Zettel ID format' };
        }
        
        return { valid: true };
    }

    sanitizeInput(input) {
        if (typeof input !== 'string') return '';
        
        return input
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;')
            .replace(/'/g, '&#x27;')
            .replace(/\//g, '&#x2F;');
    }

    // Modal management
    showModal(modalId) {
        // Mock implementation
        return { modalId, visible: true };
    }

    closeModal(modal) {
        // Mock implementation
        return { modal, visible: false };
    }

    // Error handling
    showError(message) {
        console.error('Error:', message);
        return { type: 'error', message };
    }

    showSuccess(message) {
        console.log('Success:', message);
        return { type: 'success', message };
    }
}

// Mock DOM elements and methods that tests might need
const mockDocument = {
    createElement: (tagName) => ({
        tagName: tagName.toUpperCase(),
        classList: {
            add: function() {},
            remove: function() {},
            contains: () => false,
            toggle: function() {}
        },
        addEventListener: function() {},
        removeEventListener: function() {},
        setAttribute: function() {},
        getAttribute: () => null,
        innerHTML: '',
        textContent: '',
        style: {},
        dataset: {}
    }),
    
    getElementById: (id) => mockDocument.createElement('div'),
    querySelector: (selector) => mockDocument.createElement('div'),
    querySelectorAll: (selector) => [],
    
    addEventListener: function() {},
    removeEventListener: function() {}
};

// Mock window object
const mockWindow = {
    localStorage: {
        getItem: () => null,
        setItem: () => {},
        removeItem: () => {},
        clear: () => {}
    },
    
    setTimeout: (fn, delay) => setTimeout(fn, delay),
    clearTimeout: (id) => clearTimeout(id),
    
    MathJax: {
        typesetPromise: () => Promise.resolve()
    }
};

// Make mocks available globally for tests
if (typeof window === 'undefined') {
    global.document = mockDocument;
    global.window = mockWindow;
}

// Export for use in tests
window.LearningSystemTest = LearningSystemTest;