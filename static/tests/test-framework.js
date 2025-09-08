/**
 * Minimal Test Framework for Frontend Testing
 */
class TestFramework {
    constructor() {
        this.testSuites = new Map();
        this.currentSuite = null;
        this.results = {
            total: 0,
            passed: 0,
            failed: 0,
            pending: 0
        };
    }

    describe(suiteName, suiteCallback) {
        this.currentSuite = {
            name: suiteName,
            tests: [],
            beforeEach: null,
            afterEach: null,
            beforeAll: null,
            afterAll: null
        };
        
        // Execute the suite definition
        suiteCallback();
        
        this.testSuites.set(suiteName, this.currentSuite);
        this.currentSuite = null;
    }

    it(testName, testCallback) {
        if (!this.currentSuite) {
            throw new Error('it() must be called within a describe() block');
        }
        
        this.currentSuite.tests.push({
            name: testName,
            callback: testCallback,
            status: 'pending',
            error: null,
            duration: 0
        });
    }

    beforeEach(callback) {
        if (this.currentSuite) {
            this.currentSuite.beforeEach = callback;
        }
    }

    afterEach(callback) {
        if (this.currentSuite) {
            this.currentSuite.afterEach = callback;
        }
    }

    beforeAll(callback) {
        if (this.currentSuite) {
            this.currentSuite.beforeAll = callback;
        }
    }

    afterAll(callback) {
        if (this.currentSuite) {
            this.currentSuite.afterAll = callback;
        }
    }

    async runAllTests() {
        this.results = { total: 0, passed: 0, failed: 0, pending: 0 };
        const outputDiv = document.getElementById('test-output');
        outputDiv.innerHTML = '';

        for (const [suiteName, suite] of this.testSuites) {
            await this.runTestSuite(suite, outputDiv);
        }

        this.displaySummary(outputDiv);
    }

    async runTestSuite(suite, outputDiv) {
        const suiteDiv = document.createElement('div');
        suiteDiv.className = 'test-suite';
        suiteDiv.innerHTML = `<h2>${suite.name}</h2>`;
        outputDiv.appendChild(suiteDiv);

        // Run beforeAll
        if (suite.beforeAll) {
            try {
                await suite.beforeAll();
            } catch (error) {
                console.error('beforeAll failed:', error);
            }
        }

        for (const test of suite.tests) {
            await this.runTest(test, suite, suiteDiv);
        }

        // Run afterAll
        if (suite.afterAll) {
            try {
                await suite.afterAll();
            } catch (error) {
                console.error('afterAll failed:', error);
            }
        }
    }

    async runTest(test, suite, suiteDiv) {
        const startTime = performance.now();
        
        try {
            // Run beforeEach
            if (suite.beforeEach) {
                await suite.beforeEach();
            }

            // Run the test
            if (test.callback.constructor.name === 'AsyncFunction') {
                await test.callback();
            } else {
                test.callback();
            }

            test.status = 'pass';
            test.duration = performance.now() - startTime;
            this.results.passed++;

        } catch (error) {
            test.status = 'fail';
            test.error = error;
            test.duration = performance.now() - startTime;
            this.results.failed++;
        }

        try {
            // Run afterEach
            if (suite.afterEach) {
                await suite.afterEach();
            }
        } catch (error) {
            console.error('afterEach failed:', error);
        }

        this.results.total++;
        this.displayTestResult(test, suiteDiv);
    }

    displayTestResult(test, suiteDiv) {
        const testDiv = document.createElement('div');
        testDiv.className = `test-case ${test.status}`;
        
        const symbol = test.status === 'pass' ? '✓' : 
                      test.status === 'fail' ? '✗' : '○';
        
        testDiv.innerHTML = `${symbol} ${test.name} (${test.duration.toFixed(2)}ms)`;
        
        if (test.error) {
            const errorDiv = document.createElement('div');
            errorDiv.className = 'error-details';
            errorDiv.textContent = test.error.stack || test.error.message;
            testDiv.appendChild(errorDiv);
        }
        
        suiteDiv.appendChild(testDiv);
    }

    displaySummary(outputDiv) {
        const summaryDiv = document.createElement('div');
        summaryDiv.className = 'summary';
        
        const passRate = this.results.total > 0 ? 
            (this.results.passed / this.results.total * 100).toFixed(1) : 0;
        
        summaryDiv.innerHTML = `
            <h3>Test Results</h3>
            <p><strong>Total:</strong> ${this.results.total}</p>
            <p><strong>Passed:</strong> ${this.results.passed}</p>
            <p><strong>Failed:</strong> ${this.results.failed}</p>
            <p><strong>Pass Rate:</strong> ${passRate}%</p>
        `;
        
        outputDiv.appendChild(summaryDiv);
    }
}

/**
 * Assertion Library
 */
class Expect {
    constructor(actual) {
        this.actual = actual;
        this.isNot = false;
    }

    get not() {
        const newExpect = new Expect(this.actual);
        newExpected.isNot = !this.isNot;
        return newExpect;
    }

    toBe(expected) {
        const passed = this.actual === expected;
        if (passed === this.isNot) {
            throw new Error(`Expected ${this.actual} ${this.isNot ? 'not ' : ''}to be ${expected}`);
        }
    }

    toEqual(expected) {
        const passed = JSON.stringify(this.actual) === JSON.stringify(expected);
        if (passed === this.isNot) {
            throw new Error(`Expected ${JSON.stringify(this.actual)} ${this.isNot ? 'not ' : ''}to equal ${JSON.stringify(expected)}`);
        }
    }

    toBeTrue() {
        this.toBe(true);
    }

    toBeFalse() {
        this.toBe(false);
    }

    toBeNull() {
        this.toBe(null);
    }

    toBeUndefined() {
        this.toBe(undefined);
    }

    toBeDefined() {
        if ((this.actual !== undefined) === this.isNot) {
            throw new Error(`Expected value ${this.isNot ? 'not ' : ''}to be defined`);
        }
    }

    toContain(expected) {
        const passed = this.actual && this.actual.includes && this.actual.includes(expected);
        if (passed === this.isNot) {
            throw new Error(`Expected ${this.actual} ${this.isNot ? 'not ' : ''}to contain ${expected}`);
        }
    }

    toMatch(regex) {
        const passed = regex.test(this.actual);
        if (passed === this.isNot) {
            throw new Error(`Expected ${this.actual} ${this.isNot ? 'not ' : ''}to match ${regex}`);
        }
    }

    toHaveProperty(property) {
        const passed = this.actual && property in this.actual;
        if (passed === this.isNot) {
            throw new Error(`Expected object ${this.isNot ? 'not ' : ''}to have property ${property}`);
        }
    }

    toHaveLength(length) {
        const passed = this.actual && this.actual.length === length;
        if (passed === this.isNot) {
            throw new Error(`Expected ${this.actual} ${this.isNot ? 'not ' : ''}to have length ${length}, but was ${this.actual ? this.actual.length : 'undefined'}`);
        }
    }

    toThrow(expectedError) {
        let threwError = false;
        let actualError = null;

        try {
            if (typeof this.actual === 'function') {
                this.actual();
            }
        } catch (error) {
            threwError = true;
            actualError = error;
        }

        if (threwError === this.isNot) {
            throw new Error(`Expected function ${this.isNot ? 'not ' : ''}to throw`);
        }

        if (!this.isNot && expectedError && actualError.message !== expectedError) {
            throw new Error(`Expected function to throw "${expectedError}", but threw "${actualError.message}"`);
        }
    }

    toBeInstanceOf(constructor) {
        const passed = this.actual instanceof constructor;
        if (passed === this.isNot) {
            throw new Error(`Expected ${this.actual} ${this.isNot ? 'not ' : ''}to be instance of ${constructor.name}`);
        }
    }
}

/**
 * Mock and Spy utilities
 */
class Mock {
    static fn(implementation) {
        const mockFn = implementation || (() => {});
        mockFn.calls = [];
        mockFn.results = [];
        
        const spy = function(...args) {
            spy.calls.push(args);
            try {
                const result = mockFn.apply(this, args);
                spy.results.push({ type: 'return', value: result });
                return result;
            } catch (error) {
                spy.results.push({ type: 'throw', value: error });
                throw error;
            }
        };
        
        spy.calls = mockFn.calls;
        spy.results = mockFn.results;
        spy.mockReturnValue = (value) => {
            mockFn = () => value;
            return spy;
        };
        spy.mockResolvedValue = (value) => {
            mockFn = () => Promise.resolve(value);
            return spy;
        };
        spy.mockRejectedValue = (error) => {
            mockFn = () => Promise.reject(error);
            return spy;
        };
        
        return spy;
    }

    static spyOn(object, method) {
        const originalMethod = object[method];
        const spy = this.fn(originalMethod.bind(object));
        object[method] = spy;
        
        spy.mockRestore = () => {
            object[method] = originalMethod;
        };
        
        return spy;
    }
}

// Global test framework instance
const TestRunner = new TestFramework();

// Global functions
const describe = TestRunner.describe.bind(TestRunner);
const it = TestRunner.it.bind(TestRunner);
const beforeEach = TestRunner.beforeEach.bind(TestRunner);
const afterEach = TestRunner.afterEach.bind(TestRunner);
const beforeAll = TestRunner.beforeAll.bind(TestRunner);
const afterAll = TestRunner.afterAll.bind(TestRunner);
const expect = (actual) => new Expect(actual);
const mock = Mock;