/**
 * Test Utilities for Frontend Testing
 */
class TestUtils {
    /**
     * Create a mock DOM element
     */
    static createMockElement(tagName = 'div', attributes = {}) {
        const element = document.createElement(tagName);
        Object.entries(attributes).forEach(([key, value]) => {
            element.setAttribute(key, value);
        });
        return element;
    }

    /**
     * Create a mock event
     */
    static createMockEvent(type, properties = {}) {
        const event = new Event(type, { bubbles: true, cancelable: true });
        Object.assign(event, properties);
        return event;
    }

    /**
     * Wait for a specified amount of time
     */
    static wait(ms = 0) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }

    /**
     * Wait for a condition to be true
     */
    static async waitFor(condition, timeout = 1000, interval = 10) {
        const startTime = Date.now();
        
        while (Date.now() - startTime < timeout) {
            if (await condition()) {
                return true;
            }
            await this.wait(interval);
        }
        
        throw new Error(`Condition not met within ${timeout}ms`);
    }

    /**
     * Mock fetch API
     */
    static mockFetch(responses = {}) {
        const originalFetch = global.fetch;
        
        const mockFetch = jest.fn((url, options) => {
            const response = responses[url] || responses.default;
            
            if (!response) {
                return Promise.reject(new Error(`No mock response for ${url}`));
            }
            
            return Promise.resolve({
                ok: response.ok !== false,
                status: response.status || 200,
                statusText: response.statusText || 'OK',
                json: () => Promise.resolve(response.data || response),
                text: () => Promise.resolve(JSON.stringify(response.data || response))
            });
        });
        
        global.fetch = mockFetch;
        
        return {
            mockFetch,
            restore: () => {
                global.fetch = originalFetch;
            }
        };
    }

    /**
     * Create a test fixture - a clean DOM container
     */
    static createFixture(innerHTML = '') {
        const fixture = document.createElement('div');
        fixture.id = 'test-fixture';
        fixture.innerHTML = innerHTML;
        document.body.appendChild(fixture);
        
        return {
            element: fixture,
            cleanup: () => {
                if (fixture.parentNode) {
                    fixture.parentNode.removeChild(fixture);
                }
            }
        };
    }

    /**
     * Fire a DOM event on an element
     */
    static fireEvent(element, eventType, eventProperties = {}) {
        const event = this.createMockEvent(eventType, eventProperties);
        element.dispatchEvent(event);
        return event;
    }

    /**
     * Get element by test ID attribute
     */
    static getByTestId(testId, container = document) {
        return container.querySelector(`[data-testid="${testId}"]`);
    }

    /**
     * Get all elements by test ID attribute
     */
    static getAllByTestId(testId, container = document) {
        return Array.from(container.querySelectorAll(`[data-testid="${testId}"]`));
    }

    /**
     * Mock local storage
     */
    static mockLocalStorage() {
        const storage = {};
        
        const mockLocalStorage = {
            getItem: jest.fn(key => storage[key] || null),
            setItem: jest.fn((key, value) => {
                storage[key] = value;
            }),
            removeItem: jest.fn(key => {
                delete storage[key];
            }),
            clear: jest.fn(() => {
                Object.keys(storage).forEach(key => delete storage[key]);
            }),
            get length() {
                return Object.keys(storage).length;
            },
            key: jest.fn(index => {
                const keys = Object.keys(storage);
                return keys[index] || null;
            })
        };
        
        const originalLocalStorage = global.localStorage;
        global.localStorage = mockLocalStorage;
        
        return {
            mockLocalStorage,
            storage,
            restore: () => {
                global.localStorage = originalLocalStorage;
            }
        };
    }

    /**
     * Mock console methods
     */
    static mockConsole() {
        const originalConsole = { ...console };
        const mockedMethods = {};
        
        ['log', 'error', 'warn', 'info', 'debug'].forEach(method => {
            mockedMethods[method] = [];
            console[method] = jest.fn((...args) => {
                mockedMethods[method].push(args);
            });
        });
        
        return {
            mockedMethods,
            restore: () => {
                Object.assign(console, originalConsole);
            }
        };
    }

    /**
     * Assert element has class
     */
    static assertHasClass(element, className) {
        if (!element.classList.contains(className)) {
            throw new Error(`Element does not have class "${className}"`);
        }
    }

    /**
     * Assert element is visible
     */
    static assertVisible(element) {
        const style = window.getComputedStyle(element);
        if (style.display === 'none' || style.visibility === 'hidden' || style.opacity === '0') {
            throw new Error('Element is not visible');
        }
    }

    /**
     * Assert element is hidden
     */
    static assertHidden(element) {
        const style = window.getComputedStyle(element);
        if (style.display !== 'none' && style.visibility !== 'hidden' && style.opacity !== '0') {
            throw new Error('Element is not hidden');
        }
    }

    /**
     * Simulate user typing in an input
     */
    static async typeIntoInput(input, text, { delay = 10 } = {}) {
        input.focus();
        
        for (let i = 0; i < text.length; i++) {
            const char = text[i];
            input.value += char;
            
            this.fireEvent(input, 'input', { target: { value: input.value } });
            this.fireEvent(input, 'keypress', { key: char, charCode: char.charCodeAt(0) });
            
            if (delay > 0) {
                await this.wait(delay);
            }
        }
        
        this.fireEvent(input, 'change', { target: { value: input.value } });
    }

    /**
     * Simulate a click with proper event sequence
     */
    static click(element) {
        this.fireEvent(element, 'mousedown');
        this.fireEvent(element, 'mouseup');
        this.fireEvent(element, 'click');
    }

    /**
     * Create a spy for object methods
     */
    static createSpy(object, methodName) {
        const originalMethod = object[methodName];
        const calls = [];
        
        object[methodName] = function(...args) {
            calls.push(args);
            return originalMethod.apply(this, args);
        };
        
        return {
            calls,
            restore: () => {
                object[methodName] = originalMethod;
            }
        };
    }

    /**
     * Generate test data
     */
    static generateTestCard(overrides = {}) {
        return {
            id: 1,
            zettel_id: '1.1',
            title: 'Test Card',
            content: 'This is a test card content',
            links: [],
            topics: ['test'],
            ...overrides
        };
    }

    static generateTestCards(count = 3) {
        return Array.from({ length: count }, (_, i) => 
            this.generateTestCard({ 
                id: i + 1, 
                zettel_id: `1.${i + 1}`,
                title: `Test Card ${i + 1}` 
            })
        );
    }

    /**
     * Mock API responses for common endpoints
     */
    static setupApiMocks() {
        const responses = {
            '/api/cards': {
                data: this.generateTestCards(5)
            },
            '/api/topics': {
                data: [
                    { id: 1, name: 'Mathematics', description: 'Math topics' },
                    { id: 2, name: 'Science', description: 'Science topics' }
                ]
            },
            '/api/review/due': {
                data: { due_count: 3, cards: this.generateTestCards(3) }
            }
        };
        
        return this.mockFetch(responses);
    }
}

// Export for use in tests
window.TestUtils = TestUtils;