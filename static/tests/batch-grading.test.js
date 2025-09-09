/**
 * Frontend Tests for Batch Grading and Parallel Processing
 */

describe('Batch Answer Submission', () => {
    let app;
    let mockApiCall;
    
    beforeEach(() => {
        app = new LearningSystemTest();
        mockApiCall = app.apiCall;
        
        // Mock review session
        app.reviewSession = {
            sessionId: 'test-session-123',
            dueCards: [
                { id: 'card-1', title: 'Test Card', content: 'Test content' }
            ],
            currentCardIndex: 0,
            totalCards: 1,
            correctAnswers: 0,
            totalQuestions: 0
        };
        
        app.currentQuiz = {
            card: { id: 'card-1', title: 'Test Card', content: 'Test content' },
            questions: [
                {
                    question: 'What is parallel processing?',
                    question_type: 'short_answer',
                    correct_answer: 'Multiple tasks executing simultaneously'
                },
                {
                    question: 'Which is faster for independent tasks?',
                    question_type: 'multiple_choice',
                    options: ['A) Sequential', 'B) Parallel', 'C) Both same'],
                    correct_answer: 'B) Parallel'
                },
                {
                    question: 'What is a benefit of batch processing?',
                    question_type: 'short_answer',
                    correct_answer: 'Reduced API calls and improved performance'
                }
            ]
        };
    });

    describe('Batch Submission Logic', () => {
        it('should collect all answers before submission', () => {
            const answers = [
                'Multiple tasks executing simultaneously',
                'B) Parallel',
                'Reduced API calls and improved performance'
            ];

            // Simulate user entering answers
            answers.forEach((answer, index) => {
                app.setQuestionAnswer(index, answer);
            });

            const collectedAnswers = app.getAllAnswers();
            
            expect(collectedAnswers.length).toBe(3);
            expect(collectedAnswers[0]).toBe('Multiple tasks executing simultaneously');
            expect(collectedAnswers[1]).toBe('B) Parallel');
            expect(collectedAnswers[2]).toBe('Reduced API calls and improved performance');
        });

        it('should create proper batch request structure', () => {
            const answers = ['Answer 1', 'Answer 2', 'Answer 3'];
            
            const batchRequest = app.createBatchAnswerRequest(answers);
            
            expect(batchRequest.answers).toBeDefined();
            expect(batchRequest.answers.length).toBe(3);
            expect(batchRequest.answers[0].question_index).toBe(0);
            expect(batchRequest.answers[0].answer).toBe('Answer 1');
            expect(batchRequest.answers[1].question_index).toBe(1);
            expect(batchRequest.answers[1].answer).toBe('Answer 2');
            expect(batchRequest.answers[2].question_index).toBe(2);
            expect(batchRequest.answers[2].answer).toBe('Answer 3');
        });

        it('should handle empty answer validation', () => {
            const answers = ['Answer 1', '', 'Answer 3'];
            
            const validation = app.validateAnswersForBatch(answers);
            
            expect(validation.isValid).toBe(false);
            expect(validation.errors).toContain('Question 2 has no answer');
            expect(validation.emptyAnswers).toEqual([1]);
        });

        it('should validate all answers are provided', () => {
            const completeAnswers = ['Answer 1', 'Answer 2', 'Answer 3'];
            const incompleteAnswers = ['Answer 1', 'Answer 2'];
            
            expect(app.validateAnswersForBatch(completeAnswers).isValid).toBe(true);
            expect(app.validateAnswersForBatch(incompleteAnswers).isValid).toBe(false);
        });
    });

    describe('API Integration', () => {
        it('should call batch endpoint with correct parameters', async () => {
            const answers = ['Answer 1', 'Answer 2', 'Answer 3'];
            
            // Mock successful batch response
            app.apiCall = jest.fn().mockResolvedValue({
                success: true,
                data: [
                    { question_id: '1', is_correct: true, feedback: 'Correct!', suggested_rating: 4 },
                    { question_id: '2', is_correct: true, feedback: 'Good!', suggested_rating: 3 },
                    { question_id: '3', is_correct: false, feedback: 'Try again', suggested_rating: 2 }
                ]
            });

            const result = await app.submitAllAnswersBatch(answers);
            
            expect(app.apiCall).toHaveBeenCalledWith(
                `/review/session/${app.reviewSession.sessionId}/answers/${app.currentQuiz.card.id}/batch`,
                expect.objectContaining({
                    method: 'POST',
                    body: expect.stringContaining('answers')
                })
            );

            expect(result.success).toBe(true);
            expect(result.data.length).toBe(3);
        });

        it('should handle batch submission errors gracefully', async () => {
            const answers = ['Answer 1', 'Answer 2'];
            
            app.apiCall = jest.fn().mockRejectedValue(new Error('Network error'));

            const result = await app.submitAllAnswersBatch(answers);
            
            expect(result.success).toBe(false);
            expect(result.error).toContain('Network error');
        });

        it('should fallback to sequential submission on batch failure', async () => {
            const answers = ['Answer 1', 'Answer 2'];
            
            // Mock batch failure followed by successful individual calls
            app.apiCall = jest.fn()
                .mockRejectedValueOnce(new Error('Batch failed'))
                .mockResolvedValue({
                    success: true,
                    data: { is_correct: true, feedback: 'Good', suggested_rating: 3 }
                });

            const result = await app.submitAllAnswersWithFallback(answers);
            
            expect(app.apiCall).toHaveBeenCalledTimes(3); // 1 batch + 2 individual
            expect(result.success).toBe(true);
            expect(result.fallbackUsed).toBe(true);
        });
    });

    describe('Parallel Processing Mode (Phase 2)', () => {
        it('should support parallel processing mode selection', () => {
            app.setProcessingMode('parallel');
            
            expect(app.processingMode).toBe('parallel');
            expect(app.isParallelProcessingEnabled()).toBe(true);
        });

        it('should create parallel request with processing mode', () => {
            app.setProcessingMode('parallel');
            const answers = ['Answer 1', 'Answer 2'];
            
            const parallelRequest = app.createParallelAnswerRequest(answers);
            
            expect(parallelRequest.processing_mode).toBe('parallel');
            expect(parallelRequest.answers).toBeDefined();
            expect(parallelRequest.answers.length).toBe(2);
        });

        it('should call parallel endpoint when enabled', async () => {
            app.setProcessingMode('parallel');
            const answers = ['Answer 1', 'Answer 2'];
            
            app.apiCall = jest.fn().mockResolvedValue({
                success: true,
                data: [
                    { question_id: '1', is_correct: true, feedback: 'Correct!', suggested_rating: 4 },
                    { question_id: '2', is_correct: true, feedback: 'Good!', suggested_rating: 3 }
                ],
                metrics: {
                    total_processing_time_ms: 150,
                    parallel_tasks_spawned: 2,
                    concurrent_execution_count: 2
                }
            });

            const result = await app.submitAllAnswersParallel(answers);
            
            expect(app.apiCall).toHaveBeenCalledWith(
                expect.stringContaining('/parallel'),
                expect.objectContaining({
                    method: 'POST',
                    body: expect.stringContaining('processing_mode')
                })
            );

            expect(result.metrics).toBeDefined();
            expect(result.metrics.parallel_tasks_spawned).toBe(2);
        });

        it('should fallback from parallel to batch processing', async () => {
            app.setProcessingMode('parallel');
            const answers = ['Answer 1', 'Answer 2'];
            
            // Mock parallel failure, batch success
            app.apiCall = jest.fn()
                .mockRejectedValueOnce(new Error('Parallel endpoint not available'))
                .mockResolvedValue({
                    success: true,
                    data: [
                        { question_id: '1', is_correct: true, feedback: 'Good', suggested_rating: 3 },
                        { question_id: '2', is_correct: true, feedback: 'Good', suggested_rating: 3 }
                    ]
                });

            const result = await app.submitAllAnswersWithAllFallbacks(answers);
            
            expect(app.apiCall).toHaveBeenCalledTimes(2); // parallel then batch
            expect(result.success).toBe(true);
            expect(result.processingMode).toBe('batch'); // Fell back to batch
        });
    });

    describe('Performance Monitoring', () => {
        it('should track submission timing', async () => {
            const answers = ['Answer 1', 'Answer 2'];
            
            app.apiCall = jest.fn().mockResolvedValue({
                success: true,
                data: [
                    { question_id: '1', is_correct: true, feedback: 'Good', suggested_rating: 3 },
                    { question_id: '2', is_correct: true, feedback: 'Good', suggested_rating: 3 }
                ]
            });

            const startTime = performance.now();
            const result = await app.submitAllAnswersBatch(answers);
            const endTime = performance.now();
            
            expect(result.timing).toBeDefined();
            expect(result.timing.submissionTime).toBeGreaterThan(0);
            expect(result.timing.submissionTime).toBeLessThan(endTime - startTime + 10); // Allow some margin
        });

        it('should compare batch vs sequential performance', async () => {
            const answers = ['Answer 1', 'Answer 2', 'Answer 3'];
            
            app.apiCall = jest.fn().mockResolvedValue({
                success: true,
                data: { is_correct: true, feedback: 'Good', suggested_rating: 3 }
            });

            // Simulate sequential timing
            const sequentialStart = performance.now();
            await app.submitAllAnswersSequential(answers);
            const sequentialTime = performance.now() - sequentialStart;

            // Reset mock for batch
            app.apiCall = jest.fn().mockResolvedValue({
                success: true,
                data: [
                    { question_id: '1', is_correct: true, feedback: 'Good', suggested_rating: 3 },
                    { question_id: '2', is_correct: true, feedback: 'Good', suggested_rating: 3 },
                    { question_id: '3', is_correct: true, feedback: 'Good', suggested_rating: 3 }
                ]
            });

            const batchStart = performance.now();
            await app.submitAllAnswersBatch(answers);
            const batchTime = performance.now() - batchStart;

            // Batch should generally be faster (fewer API calls)
            console.log(`Sequential: ${sequentialTime}ms, Batch: ${batchTime}ms`);
            
            // This assertion may not hold in mocked tests, but provides measurement
            expect(batchTime).toBeLessThan(sequentialTime * 2); // At least not worse than 2x slower
        });
    });

    describe('User Experience', () => {
        it('should show loading state during batch submission', async () => {
            const answers = ['Answer 1', 'Answer 2'];
            
            // Mock delayed response
            app.apiCall = jest.fn().mockImplementation(() => 
                new Promise(resolve => setTimeout(() => resolve({
                    success: true,
                    data: [
                        { question_id: '1', is_correct: true, feedback: 'Good', suggested_rating: 3 },
                        { question_id: '2', is_correct: true, feedback: 'Good', suggested_rating: 3 }
                    ]
                }), 100))
            );

            const submissionPromise = app.submitAllAnswersBatch(answers);
            
            // Check loading state is active
            expect(app.isBatchSubmissionInProgress()).toBe(true);
            
            await submissionPromise;
            
            // Check loading state is cleared
            expect(app.isBatchSubmissionInProgress()).toBe(false);
        });

        it('should display batch results with proper formatting', () => {
            const mockResults = [
                { question_id: '1', is_correct: true, feedback: 'Excellent answer!', suggested_rating: 4 },
                { question_id: '2', is_correct: false, feedback: 'Close, but missing key points', suggested_rating: 2 },
                { question_id: '3', is_correct: true, feedback: 'Good understanding', suggested_rating: 3 }
            ];

            const displayData = app.formatBatchResults(mockResults);
            
            expect(displayData.length).toBe(3);
            expect(displayData[0].questionNumber).toBe(1);
            expect(displayData[0].status).toBe('correct');
            expect(displayData[0].feedback).toBe('Excellent answer!');
            expect(displayData[0].suggestedRating).toBe(4);
            
            expect(displayData[1].status).toBe('incorrect');
            expect(displayData[2].status).toBe('correct');
        });

        it('should calculate overall performance metrics', () => {
            const mockResults = [
                { question_id: '1', is_correct: true, feedback: 'Good', suggested_rating: 4 },
                { question_id: '2', is_correct: false, feedback: 'Try again', suggested_rating: 2 },
                { question_id: '3', is_correct: true, feedback: 'Good', suggested_rating: 3 },
                { question_id: '4', is_correct: true, feedback: 'Excellent', suggested_rating: 4 }
            ];

            const metrics = app.calculateBatchMetrics(mockResults);
            
            expect(metrics.totalQuestions).toBe(4);
            expect(metrics.correctAnswers).toBe(3);
            expect(metrics.accuracy).toBe(75);
            expect(metrics.averageRating).toBe(3.25); // (4+2+3+4)/4
            expect(metrics.suggestedFinalRating).toBe(3); // Rounded average
        });

        it('should enable batch answer editing before submission', () => {
            const answers = ['Answer 1', 'Answer 2', 'Answer 3'];
            
            // Set initial answers
            app.setBatchAnswers(answers);
            
            // Edit specific answer
            app.editBatchAnswer(1, 'Modified Answer 2');
            
            const updatedAnswers = app.getBatchAnswers();
            
            expect(updatedAnswers[0]).toBe('Answer 1');
            expect(updatedAnswers[1]).toBe('Modified Answer 2');
            expect(updatedAnswers[2]).toBe('Answer 3');
        });

        it('should handle partial answer submission', () => {
            const partialAnswers = ['Answer 1', '', 'Answer 3', ''];
            
            const validation = app.validatePartialSubmission(partialAnswers);
            
            expect(validation.hasAnswers).toBe(true);
            expect(validation.answerCount).toBe(2);
            expect(validation.emptyCount).toBe(2);
            expect(validation.canSubmitPartial).toBe(true);
            expect(validation.filledIndices).toEqual([0, 2]);
        });
    });

    describe('Error Handling and Recovery', () => {
        it('should retry failed batch submissions', async () => {
            const answers = ['Answer 1', 'Answer 2'];
            
            app.apiCall = jest.fn()
                .mockRejectedValueOnce(new Error('Network timeout'))
                .mockRejectedValueOnce(new Error('Server error'))
                .mockResolvedValue({
                    success: true,
                    data: [
                        { question_id: '1', is_correct: true, feedback: 'Good', suggested_rating: 3 },
                        { question_id: '2', is_correct: true, feedback: 'Good', suggested_rating: 3 }
                    ]
                });

            const result = await app.submitAllAnswersBatchWithRetry(answers, 3);
            
            expect(app.apiCall).toHaveBeenCalledTimes(3);
            expect(result.success).toBe(true);
            expect(result.retriesUsed).toBe(2);
        });

        it('should provide meaningful error messages for different failure types', async () => {
            const testCases = [
                { error: new Error('Network error'), expectedMessage: 'Network connection failed' },
                { error: { status: 400 }, expectedMessage: 'Invalid request data' },
                { error: { status: 500 }, expectedMessage: 'Server error occurred' },
                { error: { status: 404 }, expectedMessage: 'Endpoint not found' }
            ];

            for (const testCase of testCases) {
                app.apiCall = jest.fn().mockRejectedValue(testCase.error);
                
                const result = await app.submitAllAnswersBatch(['Answer']);
                
                expect(result.success).toBe(false);
                expect(result.userFriendlyError).toContain(testCase.expectedMessage);
            }
        });

        it('should preserve user answers during error recovery', async () => {
            const answers = ['Important answer 1', 'Important answer 2'];
            
            app.apiCall = jest.fn().mockRejectedValue(new Error('Submission failed'));
            
            await app.submitAllAnswersBatch(answers);
            
            // Answers should still be available for retry
            const preservedAnswers = app.getLastSubmissionAnswers();
            
            expect(preservedAnswers).toEqual(answers);
            expect(app.canRetryLastSubmission()).toBe(true);
        });
    });

    describe('Progressive Enhancement', () => {
        it('should degrade gracefully when batch endpoint is unavailable', async () => {
            const answers = ['Answer 1', 'Answer 2'];
            
            // Mock 404 for batch endpoint
            app.apiCall = jest.fn()
                .mockRejectedValueOnce({ status: 404, message: 'Not Found' })
                .mockResolvedValue({
                    success: true,
                    data: { is_correct: true, feedback: 'Good', suggested_rating: 3 }
                });

            const result = await app.submitAllAnswersWithGracefulDegradation(answers);
            
            expect(result.success).toBe(true);
            expect(result.processingMode).toBe('sequential');
            expect(result.degradationReason).toContain('Batch endpoint not available');
        });

        it('should adapt to browser capabilities', () => {
            // Mock older browser without Promise.allSettled
            const originalPromiseAllSettled = Promise.allSettled;
            delete Promise.allSettled;
            
            const hasModernPromiseSupport = app.checkModernPromiseSupport();
            
            expect(hasModernPromiseSupport).toBe(false);
            expect(app.shouldUsePolyfill()).toBe(true);
            
            // Restore
            Promise.allSettled = originalPromiseAllSettled;
        });
    });
});