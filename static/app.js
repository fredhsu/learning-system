class LearningSystem {
    constructor() {
        this.baseURL = '/api';
        this.currentView = 'cards';
        this.currentQuiz = null;
        this.init();
    }

    async init() {
        this.setupEventListeners();
        this.switchView('cards');
        await this.loadCards();
        await this.loadTopics();
    }

    setupEventListeners() {
        // Navigation
        document.getElementById('nav-cards').addEventListener('click', () => this.switchView('cards'));
        document.getElementById('nav-review').addEventListener('click', () => this.switchView('review'));
        document.getElementById('nav-topics').addEventListener('click', () => this.switchView('topics'));

        // Card management
        document.getElementById('create-card-btn').addEventListener('click', () => this.showModal('create-card-modal'));
        document.getElementById('create-card-form').addEventListener('submit', (e) => this.handleCreateCard(e));

        // Topic management
        document.getElementById('create-topic-btn').addEventListener('click', () => this.showModal('create-topic-modal'));
        document.getElementById('create-topic-form').addEventListener('submit', (e) => this.handleCreateTopic(e));

        // Modal close buttons
        document.querySelectorAll('.close').forEach(closeBtn => {
            closeBtn.addEventListener('click', (e) => this.closeModal(e.target.closest('.modal')));
        });

        // Click outside modal to close
        document.querySelectorAll('.modal').forEach(modal => {
            modal.addEventListener('click', (e) => {
                if (e.target === modal) this.closeModal(modal);
            });
        });
    }

    switchView(viewName) {
        // Update navigation
        document.querySelectorAll('.nav-btn').forEach(btn => btn.classList.remove('active'));
        document.getElementById(`nav-${viewName}`).classList.add('active');

        // Update views
        document.querySelectorAll('.view').forEach(view => view.classList.remove('active'));
        document.getElementById(`${viewName}-view`).classList.add('active');

        this.currentView = viewName;

        // Load view-specific data
        if (viewName === 'review') {
            this.loadReviewSession();
        }
    }

    showModal(modalId) {
        document.getElementById(modalId).classList.add('active');
    }

    closeModal(modal) {
        modal.classList.remove('active');
    }

    showLoading() {
        document.getElementById('loading').style.display = 'block';
    }

    hideLoading() {
        document.getElementById('loading').style.display = 'none';
    }

    showError(message) {
        const toast = document.getElementById('error-toast');
        toast.textContent = message;
        toast.classList.add('show');
        setTimeout(() => toast.classList.remove('show'), 5000);
    }

    async apiCall(endpoint, options = {}) {
        this.showLoading();
        try {
            const response = await fetch(`${this.baseURL}${endpoint}`, {
                headers: {
                    'Content-Type': 'application/json',
                    ...options.headers
                },
                ...options
            });

            const data = await response.json();
            
            if (!response.ok) {
                throw new Error(data.error || 'API request failed');
            }

            return data.data;
        } catch (error) {
            this.showError(error.message);
            throw error;
        } finally {
            this.hideLoading();
        }
    }

    async loadCards() {
        try {
            const cards = await this.apiCall('/cards');
            this.renderCards(cards);
        } catch (error) {
            this.showError('Failed to load cards');
        }
    }

    renderCards(cards) {
        const container = document.getElementById('cards-list');
        
        if (cards.length === 0) {
            container.innerHTML = `
                <div class="empty-state">
                    <h3>No cards yet</h3>
                    <p>Create your first knowledge card to get started!</p>
                </div>
            `;
            return;
        }

        container.innerHTML = cards.map(card => `
            <div class="card" data-id="${card.id}">
                <div class="card-content">${card.content}</div>
                <div class="card-meta">
                    <span>Created: ${new Date(card.creation_date).toLocaleDateString()}</span>
                    <span>Reviews: ${card.reps}</span>
                    <span>State: ${card.state}</span>
                    ${card.next_review ? `<span>Next: ${new Date(card.next_review).toLocaleDateString()}</span>` : ''}
                </div>
            </div>
        `).join('');

        // Add MathJax rendering if available
        if (window.MathJax) {
            MathJax.typesetPromise([container]).catch((err) => console.log(err.message));
        }
    }

    async loadTopics() {
        try {
            const topics = await this.apiCall('/topics');
            this.renderTopics(topics);
        } catch (error) {
            this.showError('Failed to load topics');
        }
    }

    renderTopics(topics) {
        const container = document.getElementById('topics-list');
        
        if (topics.length === 0) {
            container.innerHTML = `
                <div class="empty-state">
                    <h3>No topics yet</h3>
                    <p>Create your first topic to organize your cards!</p>
                </div>
            `;
            return;
        }

        container.innerHTML = topics.map(topic => `
            <div class="card">
                <h3>${topic.name}</h3>
                ${topic.description ? `<p>${topic.description}</p>` : ''}
            </div>
        `).join('');
    }

    async handleCreateCard(e) {
        e.preventDefault();
        
        const content = document.getElementById('card-content').value;
        const topicsText = document.getElementById('card-topics').value;
        const linksText = document.getElementById('card-links').value;

        // For now, we'll create topics if they don't exist
        const topicNames = topicsText.split(',').map(t => t.trim()).filter(t => t);
        const topic_ids = []; // We'd need to resolve topic names to IDs in a real implementation

        const cardData = {
            content,
            topic_ids,
            links: linksText ? linksText.split(',').map(l => l.trim()) : null
        };

        try {
            await this.apiCall('/cards', {
                method: 'POST',
                body: JSON.stringify(cardData)
            });

            // Reset form and close modal
            e.target.reset();
            this.closeModal(document.getElementById('create-card-modal'));
            
            // Reload cards
            await this.loadCards();
        } catch (error) {
            this.showError('Failed to create card');
        }
    }

    async handleCreateTopic(e) {
        e.preventDefault();
        
        const name = document.getElementById('topic-name').value;
        const description = document.getElementById('topic-description').value;

        try {
            await this.apiCall('/topics', {
                method: 'POST',
                body: JSON.stringify({ name, description: description || null })
            });

            // Reset form and close modal
            e.target.reset();
            this.closeModal(document.getElementById('create-topic-modal'));
            
            // Reload topics
            await this.loadTopics();
        } catch (error) {
            this.showError('Failed to create topic');
        }
    }

    async loadReviewSession() {
        try {
            const dueCards = await this.apiCall('/cards/due');
            
            document.getElementById('due-count').textContent = `${dueCards.length} cards due for review`;
            
            if (dueCards.length === 0) {
                document.getElementById('no-reviews').style.display = 'block';
                document.getElementById('quiz-container').style.display = 'none';
            } else {
                document.getElementById('no-reviews').style.display = 'none';
                document.getElementById('quiz-container').style.display = 'block';
                await this.startQuiz(dueCards[0]);
            }
        } catch (error) {
            this.showError('Failed to load review session');
        }
    }

    async startQuiz(card) {
        try {
            // Display the card content
            document.getElementById('card-content-display').innerHTML = `
                <h3>Review Card</h3>
                <div class="card-content">${card.content}</div>
                <p class="card-meta">Next review: ${new Date(card.next_review).toLocaleDateString()}</p>
            `;

            // Generate quiz questions
            const questions = await this.apiCall(`/cards/${card.id}/quiz`);
            this.currentQuiz = { card, questions, currentQuestion: 0 };
            this.renderQuestion();
        } catch (error) {
            this.showError('Failed to generate quiz');
        }
    }

    renderQuestion() {
        const { questions, currentQuestion } = this.currentQuiz;
        const question = questions[currentQuestion];
        
        const container = document.getElementById('quiz-questions');
        
        let questionHTML = `
            <div class="question">
                <h4>Question ${currentQuestion + 1} of ${questions.length}</h4>
                <p>${question.question}</p>
        `;

        if (question.question_type === 'multiple_choice' && question.options) {
            questionHTML += `
                <div class="options">
                    ${question.options.map((option, index) => `
                        <div class="option" data-option="${String.fromCharCode(65 + index)}">
                            ${String.fromCharCode(65 + index)}. ${option}
                        </div>
                    `).join('')}
                </div>
                <button class="primary-btn" onclick="app.submitAnswer()">Submit Answer</button>
            `;
        } else {
            questionHTML += `
                <textarea class="short-answer" placeholder="Enter your answer..."></textarea>
                <button class="primary-btn" onclick="app.submitAnswer()">Submit Answer</button>
            `;
        }

        questionHTML += '</div>';
        container.innerHTML = questionHTML;

        // Add click listeners for multiple choice options
        document.querySelectorAll('.option').forEach(option => {
            option.addEventListener('click', () => {
                document.querySelectorAll('.option').forEach(o => o.classList.remove('selected'));
                option.classList.add('selected');
            });
        });
    }

    async submitAnswer() {
        const { card, questions, currentQuestion } = this.currentQuiz;
        const question = questions[currentQuestion];
        
        let answer;
        if (question.question_type === 'multiple_choice') {
            const selected = document.querySelector('.option.selected');
            if (!selected) {
                this.showError('Please select an answer');
                return;
            }
            answer = selected.dataset.option;
        } else {
            answer = document.querySelector('.short-answer').value.trim();
            if (!answer) {
                this.showError('Please enter an answer');
                return;
            }
        }

        try {
            const result = await this.apiCall(`/cards/${card.id}/quiz/answer`, {
                method: 'POST',
                body: JSON.stringify({
                    question_index: currentQuestion,
                    answer: answer
                })
            });

            this.showFeedback(result.grading, question);
        } catch (error) {
            this.showError('Failed to submit answer');
        }
    }

    showFeedback(grading, question) {
        const feedbackContainer = document.getElementById('quiz-feedback');
        
        feedbackContainer.innerHTML = `
            <div class="feedback ${grading.is_correct ? 'correct' : 'incorrect'}">
                <h4>${grading.is_correct ? 'Correct!' : 'Incorrect'}</h4>
                <p>${grading.feedback}</p>
                ${question.correct_answer ? `<p><strong>Correct answer:</strong> ${question.correct_answer}</p>` : ''}
            </div>
            <div class="rating-buttons">
                <button class="rating-btn again" onclick="app.rateCard(1)">Again</button>
                <button class="rating-btn hard" onclick="app.rateCard(2)">Hard</button>
                <button class="rating-btn good" onclick="app.rateCard(3)">Good</button>
                <button class="rating-btn easy" onclick="app.rateCard(4)">Easy</button>
            </div>
        `;
    }

    async rateCard(rating) {
        const { card } = this.currentQuiz;
        
        try {
            await this.apiCall(`/cards/${card.id}/review`, {
                method: 'POST',
                body: JSON.stringify({ rating })
            });

            // Move to next question or end quiz
            this.currentQuiz.currentQuestion++;
            if (this.currentQuiz.currentQuestion < this.currentQuiz.questions.length) {
                document.getElementById('quiz-feedback').innerHTML = '';
                this.renderQuestion();
            } else {
                // Quiz completed, load next review session
                await this.loadReviewSession();
            }
        } catch (error) {
            this.showError('Failed to record rating');
        }
    }

    // Helper function to render LaTeX with MathJax
    renderMath(element) {
        if (window.MathJax) {
            MathJax.typesetPromise([element]).catch((err) => console.log(err.message));
        }
    }
}

// Initialize the application
const app = new LearningSystem();