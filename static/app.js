class LearningSystem {
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
        this.init();
    }

    renderMarkdown(content) {
        if (typeof marked !== 'undefined') {
            return marked.parse(content);
        }
        return content.replace(/\n/g, '<br>');
    }

    createPreviewContent(content) {
        const plainTextContent = content.replace(/[#*_`~\[\]()]/g, '').substring(0, 100);
        if (typeof marked !== 'undefined') {
            return marked.parse(plainTextContent);
        }
        return plainTextContent.replace(/\n/g, '<br>');
    }

    toggleCardPreview(cardId) {
        const previewElement = document.querySelector(`[data-full-id="${cardId}"].card-preview`);
        const fullElement = document.querySelector(`[data-full-id="${cardId}"].card-full`);
        const toggleButton = document.querySelector(`[data-card-id="${cardId}"]`);
        const toggleText = toggleButton.querySelector('.toggle-text');
        const toggleIcon = toggleButton.querySelector('.toggle-icon');
        
        const isCurrentlyPreview = previewElement.style.display !== 'none';
        
        if (isCurrentlyPreview) {
            // Show full content
            previewElement.style.display = 'none';
            fullElement.style.display = 'block';
            toggleText.textContent = 'Show Less';
            toggleIcon.textContent = '▲';
            toggleButton.classList.add('expanded');
        } else {
            // Show preview
            previewElement.style.display = 'block';
            fullElement.style.display = 'none';
            toggleText.textContent = 'Show More';
            toggleIcon.textContent = '▼';
            toggleButton.classList.remove('expanded');
        }
        
        // Re-render MathJax if available
        if (window.MathJax) {
            const activeElement = isCurrentlyPreview ? fullElement : previewElement;
            MathJax.typesetPromise([activeElement]).catch((err) => console.log(err.message));
        }
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
        document.getElementById('edit-card-form').addEventListener('submit', (e) => this.handleEditCard(e));

        // Topic management
        document.getElementById('create-topic-btn').addEventListener('click', () => this.showModal('create-topic-modal'));
        document.getElementById('create-topic-form').addEventListener('submit', (e) => this.handleCreateTopic(e));

        // Review completion
        document.getElementById('new-review-btn').addEventListener('click', () => this.startNewReview());

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

        // Search functionality
        document.getElementById('card-search').addEventListener('input', (e) => this.handleSearchInput(e));
        document.getElementById('clear-search-btn').addEventListener('click', () => this.clearSearch());

        // Keyboard shortcuts
        document.addEventListener('keydown', (e) => this.handleKeyboardShortcuts(e));
        document.getElementById('keyboard-indicator').addEventListener('click', () => this.showKeyboardHelp());
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

    showLoading(message = 'Loading...') {
        const loading = document.getElementById('loading');
        const loadingText = loading.querySelector('.loading-text');
        loadingText.textContent = message;
        loading.classList.add('show');
    }

    hideLoading() {
        const loading = document.getElementById('loading');
        loading.classList.remove('show');
    }

    showSkeleton(containerId) {
        const skeleton = document.getElementById(`${containerId}-skeleton`);
        const content = document.getElementById(containerId);
        if (skeleton) {
            skeleton.style.display = 'block';
            content.style.display = 'none';
        }
    }

    hideSkeleton(containerId) {
        const skeleton = document.getElementById(`${containerId}-skeleton`);
        const content = document.getElementById(containerId);
        if (skeleton) {
            skeleton.style.display = 'none';
            content.style.display = 'block';
        }
    }

    showToast(message, type = 'info', duration = 5000) {
        const container = document.getElementById('toast-container');
        const toastId = 'toast-' + Date.now();
        
        const iconMap = {
            success: 'check-circle',
            error: 'x-circle',
            warning: 'alert-triangle',
            info: 'info'
        };
        
        const toast = document.createElement('div');
        toast.className = `toast ${type}`;
        toast.id = toastId;
        toast.innerHTML = `
            <div class="toast-icon">
                <i data-feather="${iconMap[type]}"></i>
            </div>
            <span>${message}</span>
            <button class="toast-close" onclick="app.closeToast('${toastId}')">&times;</button>
        `;
        
        container.appendChild(toast);
        
        // Initialize feather icons for the toast
        if (window.feather) {
            feather.replace();
        }
        
        // Auto-remove toast after duration
        setTimeout(() => {
            this.closeToast(toastId);
        }, duration);
        
        return toastId;
    }

    closeToast(toastId) {
        const toast = document.getElementById(toastId);
        if (toast) {
            toast.style.animation = 'toastSlideOut 0.3s ease-out forwards';
            setTimeout(() => {
                if (toast.parentNode) {
                    toast.parentNode.removeChild(toast);
                }
            }, 300);
        }
    }

    showError(message) {
        this.showToast(message, 'error');
    }

    showSuccess(message) {
        this.showToast(message, 'success');
    }

    showWarning(message) {
        this.showToast(message, 'warning');
    }

    showInfo(message) {
        this.showToast(message, 'info');
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
            
            if (!response.ok || !data.success) {
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

    handleSearchInput(event) {
        const searchQuery = event.target.value;
        const clearButton = document.getElementById('clear-search-btn');
        
        // Show/hide clear button
        if (searchQuery.trim()) {
            clearButton.style.display = 'block';
        } else {
            clearButton.style.display = 'none';
        }
        
        // Clear existing debounce timer
        if (this.searchDebounceTimer) {
            clearTimeout(this.searchDebounceTimer);
        }
        
        // Set new debounce timer
        this.searchDebounceTimer = setTimeout(() => {
            this.performSearch(searchQuery);
        }, 300);
    }

    clearSearch() {
        const searchInput = document.getElementById('card-search');
        const clearButton = document.getElementById('clear-search-btn');
        const searchInfo = document.getElementById('search-results-info');
        
        searchInput.value = '';
        clearButton.style.display = 'none';
        searchInfo.style.display = 'none';
        
        this.currentSearchQuery = '';
        this.loadCards();
    }

    async performSearch(searchQuery) {
        this.currentSearchQuery = searchQuery;
        
        try {
            const cards = await this.searchCards(searchQuery);
            this.renderCards(cards, searchQuery);
            this.updateSearchInfo(cards.length, searchQuery);
        } catch (error) {
            this.showError('Failed to search cards');
        }
    }

    async searchCards(searchQuery) {
        const endpoint = searchQuery.trim() 
            ? `/cards/search?q=${encodeURIComponent(searchQuery)}` 
            : '/cards';
        return await this.apiCall(endpoint);
    }

    updateSearchInfo(resultsCount, searchQuery) {
        const searchInfo = document.getElementById('search-results-info');
        
        if (searchQuery.trim()) {
            searchInfo.style.display = 'block';
            searchInfo.textContent = `Found ${resultsCount} card${resultsCount !== 1 ? 's' : ''} for "${searchQuery}"`;
        } else {
            searchInfo.style.display = 'none';
        }
    }

    highlightSearchTerm(text, searchTerm) {
        if (!searchTerm || !searchTerm.trim()) {
            return text;
        }
        
        const regex = new RegExp(`(${searchTerm.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'gi');
        return text.replace(regex, '<span class="search-highlight">$1</span>');
    }

    async loadCards() {
        try {
            // Show skeleton loading
            this.showSkeleton('cards-list');
            
            const cards = await this.apiCall('/cards');
            this.allCards = cards;
            
            // Hide skeleton and show cards
            this.hideSkeleton('cards-list');
            this.renderCards(cards);
        } catch (error) {
            this.hideSkeleton('cards-list');
            this.showError('Failed to load cards');
        }
    }

    renderCards(cards, searchQuery = '') {
        const container = document.getElementById('cards-list');
        
        if (cards.length === 0) {
            const emptyMessage = searchQuery.trim() 
                ? `<h3>No cards found</h3><p>No cards match your search for "${searchQuery}"</p>`
                : `<h3>No cards yet</h3><p>Create your first knowledge card to get started!</p>`;
                
            container.innerHTML = `<div class="empty-state">${emptyMessage}</div>`;
            return;
        }

        container.innerHTML = cards.map(card => {
            // Apply highlighting to card content before rendering markdown
            const highlightedContent = searchQuery.trim() 
                ? this.highlightSearchTerm(card.content, searchQuery)
                : card.content;
                
            const fullContent = this.renderMarkdown(highlightedContent);
            const previewContent = this.createPreviewContent(highlightedContent);
            const needsPreview = card.content.length > 100;
            
            return `
                <div class="card" data-id="${card.id}">
                    <div class="card-header">
                        <span class="zettel-id">${card.zettel_id}</span>
                    </div>
                    <div class="card-content-wrapper">
                        ${needsPreview ? `
                            <div class="card-content card-preview" data-full-id="${card.id}">
                                ${previewContent}
                                <span class="preview-ellipsis">...</span>
                            </div>
                            <div class="card-content card-full" data-full-id="${card.id}" style="display: none;">
                                ${fullContent}
                            </div>
                            <button class="preview-toggle-btn" data-card-id="${card.id}" onclick="app.toggleCardPreview('${card.id}')">
                                <span class="toggle-text">Show More</span>
                                <span class="toggle-icon">▼</span>
                            </button>
                        ` : `
                            <div class="card-content">${fullContent}</div>
                        `}
                    </div>
                    <div class="card-meta">
                        <div class="card-meta-item created">
                            <svg class="card-meta-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <rect x="3" y="4" width="18" height="18" rx="2" ry="2"/>
                                <line x1="16" y1="2" x2="16" y2="6"/>
                                <line x1="8" y1="2" x2="8" y2="6"/>
                                <line x1="3" y1="10" x2="21" y2="10"/>
                            </svg>
                            <span class="card-meta-label">Created</span>
                            <span class="card-meta-value">${new Date(card.creation_date).toLocaleDateString()}</span>
                        </div>
                        <div class="card-meta-item reviews">
                            <svg class="card-meta-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <path d="M9 11H5a2 2 0 0 0-2 2v5a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-5a2 2 0 0 0-2-2h-4"/>
                                <path d="M9 7l3-3 3 3"/>
                                <path d="M12 4v8"/>
                            </svg>
                            <span class="card-meta-label">Reviews</span>
                            <span class="card-meta-value">${card.reps}</span>
                        </div>
                        <div class="card-meta-item state ${card.state.toLowerCase()}">
                            <svg class="card-meta-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <circle cx="12" cy="12" r="3"/>
                                <path d="M12 1v6m0 6v6m11-7h-6m-6 0H1"/>
                            </svg>
                            <span class="card-meta-label">State</span>
                            <span class="card-meta-value">${card.state}</span>
                        </div>
                        ${card.next_review ? `
                            <div class="card-meta-item next-review">
                                <svg class="card-meta-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                    <circle cx="12" cy="12" r="10"/>
                                    <polyline points="12,6 12,12 16,14"/>
                                </svg>
                                <span class="card-meta-label">Next</span>
                                <span class="card-meta-value">${new Date(card.next_review).toLocaleDateString()}</span>
                            </div>
                        ` : ''}
                    </div>
                    <div class="card-links-section" id="card-links-${card.id}">
                        <!-- Linked cards will be loaded here -->
                    </div>
                    <div class="card-actions">
                        <button class="icon-btn edit-btn" onclick="app.editCard('${card.id}')" title="Edit card">
                            <i data-feather="edit"></i>
                        </button>
                        <button class="icon-btn delete-btn danger" onclick="app.deleteCard('${card.id}')" title="Delete card">
                            <i data-feather="trash-2"></i>
                        </button>
                    </div>
                </div>
            `;
        }).join('');

        // Add MathJax rendering if available
        if (window.MathJax) {
            MathJax.typesetPromise([container]).catch((err) => console.log(err.message));
        }
        
        // Re-initialize Feather icons for new content
        if (window.feather) {
            feather.replace();
        }
        
        // Load linked cards for each card
        cards.forEach(card => {
            this.loadLinkedCardsForCard(card.id);
        });
    }

    async loadLinkedCardsForCard(cardId) {
        try {
            // Load both forward links and backlinks
            const [linkedCards, backlinks] = await Promise.all([
                this.apiCall(`/cards/${cardId}/links`).catch(() => []),
                this.apiCall(`/cards/${cardId}/backlinks`).catch(() => [])
            ]);
            this.renderLinkedCards(cardId, linkedCards, backlinks);
        } catch (error) {
            // Silently fail - not all cards have links
        }
    }

    renderLinkedCards(cardId, linkedCards, backlinks = []) {
        const container = document.getElementById(`card-links-${cardId}`);
        if (!container || (linkedCards.length === 0 && backlinks.length === 0)) {
            return;
        }

        let linksHtml = '';
        
        // Render forward links
        if (linkedCards.length > 0) {
            linksHtml += `
                <div class="linked-cards">
                    <div class="linked-cards-header">
                        <i data-feather="link"></i>
                        <span>Linked Cards (${linkedCards.length})</span>
                    </div>
                    <div class="linked-cards-list">
                        ${linkedCards.map(linkedCard => `
                            <a href="#" class="linked-card-item" onclick="app.navigateToCard('${linkedCard.id}'); return false;">
                                <span class="linked-card-zettel">${linkedCard.zettel_id}</span>
                                <span class="linked-card-preview">${this.truncateText(linkedCard.content, 80)}</span>
                            </a>
                        `).join('')}
                    </div>
                </div>
            `;
        }

        // Render backlinks
        if (backlinks.length > 0) {
            linksHtml += `
                <div class="backlinks">
                    <div class="backlinks-header">
                        <i data-feather="corner-down-left"></i>
                        <span>Backlinks (${backlinks.length})</span>
                    </div>
                    <div class="backlinks-list">
                        ${backlinks.map(backlinkCard => `
                            <a href="#" class="backlink-item" onclick="app.navigateToCard('${backlinkCard.id}'); return false;">
                                <span class="backlink-zettel">${backlinkCard.zettel_id}</span>
                                <span class="backlink-preview">${this.truncateText(backlinkCard.content, 80)}</span>
                            </a>
                        `).join('')}
                    </div>
                </div>
            `;
        }

        container.innerHTML = linksHtml;
        
        // Re-initialize Feather icons for the linked cards section
        if (window.feather) {
            feather.replace();
        }
    }

    truncateText(text, maxLength) {
        if (text.length <= maxLength) {
            return text;
        }
        return text.substring(0, maxLength) + '...';
    }

    navigateToCard(cardId) {
        // Scroll to the card and highlight it temporarily
        const cardElement = document.querySelector(`[data-id="${cardId}"]`);
        if (cardElement) {
            cardElement.scrollIntoView({ behavior: 'smooth', block: 'center' });
            cardElement.classList.add('highlighted');
            setTimeout(() => {
                cardElement.classList.remove('highlighted');
            }, 2000);
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
        
        const zettelId = document.getElementById('card-zettel-id').value.trim();
        const content = document.getElementById('card-content').value;
        const topicsText = document.getElementById('card-topics').value;
        const linksText = document.getElementById('card-links').value;

        if (!zettelId) {
            this.showError('Zettel ID is required');
            return;
        }

        // For now, we'll create topics if they don't exist
        const topicNames = topicsText.split(',').map(t => t.trim()).filter(t => t);
        const topic_ids = []; // We'd need to resolve topic names to IDs in a real implementation

        const cardData = {
            zettel_id: zettelId,
            content,
            topic_ids,
            zettel_links: linksText ? linksText.split(',').map(l => l.trim()).filter(l => l) : null
        };

        try {
            await this.apiCall('/cards', {
                method: 'POST',
                body: JSON.stringify(cardData)
            });

            // Reset form and close modal
            e.target.reset();
            this.closeModal(document.getElementById('create-card-modal'));
            
            // Show success message
            this.showSuccess('Card created successfully!');
            
            // Reload cards with current search
            if (this.currentSearchQuery) {
                await this.performSearch(this.currentSearchQuery);
            } else {
                await this.loadCards();
            }
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
            
            // Show success message
            this.showSuccess('Topic created successfully!');
            
            // Reload topics
            await this.loadTopics();
        } catch (error) {
            this.showError('Failed to create topic');
        }
    }

    async loadReviewSession() {
        try {
            this.showLoading('Preparing review session...');
            
            // Start review session - this will generate all questions upfront
            const sessionData = await this.apiCall('/review/session/start', {
                method: 'POST'
            });
            
            // Initialize review session data
            this.reviewSession = {
                totalCards: sessionData.cards.length,
                currentCardIndex: 0,
                totalQuestions: 0,
                correctAnswers: 0,
                startTime: new Date(),
                dueCards: sessionData.cards,
                sessionId: sessionData.session_id,
                questions: sessionData.questions
            };
            
            document.getElementById('due-count').textContent = `${sessionData.cards.length} cards due for review`;
            
            if (sessionData.cards.length === 0) {
                document.getElementById('no-reviews').style.display = 'block';
                document.getElementById('quiz-container').style.display = 'none';
                document.getElementById('review-progress-bar').style.display = 'none';
                document.getElementById('remaining-count').style.display = 'none';
            } else {
                document.getElementById('no-reviews').style.display = 'none';
                document.getElementById('quiz-container').style.display = 'block';
                document.getElementById('review-progress-bar').style.display = 'block';
                document.getElementById('remaining-count').style.display = 'block';
                this.updateRemainingCount();
                await this.startQuiz(sessionData.cards[0]);
            }
        } catch (error) {
            this.showError('Failed to load review session');
        } finally {
            this.hideLoading();
        }
    }

    async startQuiz(card) {
        try {
            // Display the card content with transition
            const cardDisplay = document.getElementById('card-content-display');
            cardDisplay.classList.add('quiz-transition');
            
            cardDisplay.innerHTML = `
                <h3>Review Card</h3>
                <div class="card-content">${this.renderMarkdown(card.content)}</div>
                <p class="card-meta">Next review: ${new Date(card.next_review).toLocaleDateString()}</p>
            `;

            // Add MathJax rendering if available
            if (window.MathJax) {
                MathJax.typesetPromise([cardDisplay]).catch((err) => console.log(err.message));
            }

            // Trigger fade-in animation
            setTimeout(() => {
                cardDisplay.classList.add('fade-in');
            }, 50);

            // Use pre-generated questions from the session
            const questions = this.reviewSession.questions[card.id] || [];
            if (questions.length === 0) {
                this.showError('No questions available for this card');
                return;
            }
            
            this.currentQuiz = { card, questions, currentQuestion: 0 };
            this.updateProgressIndicators();
            this.renderQuestion();
        } catch (error) {
            this.showError('Failed to start quiz');
        }
    }

    renderQuestion() {
        const { questions, currentQuestion } = this.currentQuiz;
        const question = questions[currentQuestion];
        
        const container = document.getElementById('quiz-questions');
        container.classList.add('quiz-transition');
        
        let questionHTML = `
            <div class="question">
                <h4>Question ${currentQuestion + 1} of ${questions.length}</h4>
                <p>${question.question}</p>
        `;

        if (question.question_type === 'multiple_choice' && question.options) {
            questionHTML += `
                <div class="options">
                    ${question.options.map((option, index) => `
                        <div class="option" data-option="${String.fromCharCode(65 + index)}" data-option-text="${option}">
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

        // Update progress indicators
        this.updateProgressIndicators();

        // Trigger fade-in animation
        setTimeout(() => {
            container.classList.add('fade-in');
        }, 50);

        // Add click listeners for multiple choice options
        document.querySelectorAll('.option').forEach(option => {
            option.addEventListener('click', () => {
                document.querySelectorAll('.option').forEach(o => o.classList.remove('selected'));
                option.classList.add('selected');
                // Add visual feedback
                const questionElement = document.querySelector('.question');
                questionElement.classList.add('answering');
            });
        });

        // Add typing listener for short answer
        const shortAnswerField = document.querySelector('.short-answer');
        if (shortAnswerField) {
            shortAnswerField.addEventListener('input', () => {
                const questionElement = document.querySelector('.question');
                if (shortAnswerField.value.trim()) {
                    questionElement.classList.add('answering');
                } else {
                    questionElement.classList.remove('answering');
                }
            });
        }
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
            answer = selected.dataset.optionText;
        } else {
            answer = document.querySelector('.short-answer').value.trim();
            if (!answer) {
                this.showError('Please enter an answer');
                return;
            }
        }

        // Disable submit button to prevent double submission
        const submitButton = document.querySelector('.primary-btn');
        submitButton.disabled = true;
        submitButton.textContent = 'Submitting...';

        try {
            // Use the new session-based endpoint for context-aware grading
            const question = this.currentQuiz.questions[currentQuestion];
            const result = await this.apiCall(`/review/session/${this.reviewSession.sessionId}/answer/${card.id}`, {
                method: 'POST',
                body: JSON.stringify({
                    question_index: currentQuestion,
                    answer: answer
                })
            });

            this.showFeedback(result, question);
        } catch (error) {
            this.showError('Failed to submit answer');
            // Re-enable button on error
            submitButton.disabled = false;
            submitButton.textContent = 'Submit Answer';
        }
    }

    showFeedback(grading, question) {
        const feedbackContainer = document.getElementById('quiz-feedback');
        
        // Update statistics
        this.reviewSession.totalQuestions++;
        if (grading.is_correct) {
            this.reviewSession.correctAnswers++;
        }

        // Add visual feedback to options if multiple choice
        if (question.question_type === 'multiple_choice') {
            const selectedOption = document.querySelector('.option.selected');
            const allOptions = document.querySelectorAll('.option');
            
            // Find correct option
            allOptions.forEach(option => {
                if (option.dataset.optionText === question.correct_answer) {
                    option.classList.add('correct-flash');
                }
            });
            
            // Highlight incorrect selection if wrong
            if (selectedOption && !grading.is_correct) {
                selectedOption.classList.add('incorrect-flash');
            }
        }
        
        // Create rating name helper function
        const getRatingName = (rating) => {
            const ratingNames = { 1: 'Again', 2: 'Hard', 3: 'Good', 4: 'Easy' };
            return ratingNames[rating] || 'Unknown';
        };
        
        feedbackContainer.innerHTML = `
            <div class="feedback ${grading.is_correct ? 'correct' : 'incorrect'} feedback-transition">
                <h4>${grading.is_correct ? 'Correct!' : 'Incorrect'}</h4>
                <p>${grading.feedback}</p>
                ${question.correct_answer ? `<p><strong>Correct answer:</strong> ${question.correct_answer}</p>` : ''}
                ${grading.rating ? `<p class="suggested-rating"><strong>Suggested rating:</strong> ${getRatingName(grading.rating)} (${grading.rating})</p>` : ''}
            </div>
            <div class="rating-buttons">
                <button class="rating-btn again rating-btn-with-shortcut ${grading.rating === 1 ? 'suggested' : ''}" data-shortcut="1" onclick="app.rateCard(1)">Again</button>
                <button class="rating-btn hard rating-btn-with-shortcut ${grading.rating === 2 ? 'suggested' : ''}" data-shortcut="2" onclick="app.rateCard(2)">Hard</button>
                <button class="rating-btn good rating-btn-with-shortcut ${grading.rating === 3 ? 'suggested' : ''}" data-shortcut="3" onclick="app.rateCard(3)">Good</button>
                <button class="rating-btn easy rating-btn-with-shortcut ${grading.rating === 4 ? 'suggested' : ''}" data-shortcut="4" onclick="app.rateCard(4)">Easy</button>
            </div>
        `;
    }

    async rateCard(rating) {
        const { card, questions } = this.currentQuiz;
        
        // Store the rating for this question
        if (!this.currentQuiz.questionRatings) {
            this.currentQuiz.questionRatings = [];
        }
        this.currentQuiz.questionRatings[this.currentQuiz.currentQuestion] = rating;
        
        console.log(`Question ${this.currentQuiz.currentQuestion + 1} rated: ${rating}`);

        // Move to next question or end quiz
        this.currentQuiz.currentQuestion++;
        if (this.currentQuiz.currentQuestion < this.currentQuiz.questions.length) {
            // More questions remaining - continue without FSRS update
            document.getElementById('quiz-feedback').innerHTML = '';
            document.querySelectorAll('.option').forEach(option => {
                option.classList.remove('correct-flash', 'incorrect-flash', 'selected');
            });
            document.querySelector('.question').classList.remove('answering');
            document.getElementById('quiz-questions').classList.remove('fade-in');
            
            this.renderQuestion();
        } else {
            // All questions completed - calculate final rating and update FSRS
            try {
                const finalRating = this.calculateFinalRating(this.currentQuiz.questionRatings);
                console.log(`All questions completed. Ratings: [${this.currentQuiz.questionRatings.join(', ')}], Final rating: ${finalRating}`);
                
                await this.apiCall(`/cards/${card.id}/review`, {
                    method: 'POST',
                    body: JSON.stringify({ rating: finalRating })
                });

                // Card completed, move to next card or end session
                this.reviewSession.currentCardIndex++;
                this.updateRemainingCount();
                
                if (this.reviewSession.currentCardIndex < this.reviewSession.totalCards) {
                    // Start next card
                    const nextCard = this.reviewSession.dueCards[this.reviewSession.currentCardIndex];
                    document.getElementById('quiz-feedback').innerHTML = '';
                    document.getElementById('card-content-display').classList.remove('fade-in');
                    document.getElementById('quiz-questions').classList.remove('fade-in');
                    await this.startQuiz(nextCard);
                } else {
                    // All cards completed - show celebration
                    this.showCompletionScreen();
                }
            } catch (error) {
                this.showError('Failed to submit final rating');
                console.error('Error submitting final card rating:', error);
            }
        }
    }

    calculateFinalRating(questionRatings) {
        // Strategy: Use the average of all question ratings, rounded to nearest integer
        // This provides a balanced assessment across all questions
        
        if (!questionRatings || questionRatings.length === 0) {
            console.warn('No question ratings provided, defaulting to rating 3 (Good)');
            return 3;
        }
        
        // Filter out any undefined/null ratings
        const validRatings = questionRatings.filter(rating => rating !== undefined && rating !== null);
        
        if (validRatings.length === 0) {
            console.warn('No valid question ratings found, defaulting to rating 3 (Good)');
            return 3;
        }
        
        // Calculate average and round to nearest integer (1-4 range)
        const average = validRatings.reduce((sum, rating) => sum + rating, 0) / validRatings.length;
        const finalRating = Math.round(Math.max(1, Math.min(4, average)));
        
        console.log(`Rating calculation: [${validRatings.join(', ')}] -> average: ${average.toFixed(2)} -> final: ${finalRating}`);
        
        return finalRating;
    }

    async editCard(cardId) {
        try {
            const card = await this.apiCall(`/cards/${cardId}`);
            
            // Populate edit form
            document.getElementById('edit-card-id').value = card.id;
            document.getElementById('edit-card-zettel-id').value = card.zettel_id;
            document.getElementById('edit-card-content').value = card.content;
            
            // Get linked cards and display their Zettel IDs
            try {
                const linkedCards = await this.apiCall(`/cards/${card.id}/links`);
                if (linkedCards.length > 0) {
                    const zettelIds = linkedCards.map(linkedCard => linkedCard.zettel_id);
                    document.getElementById('edit-card-links').value = zettelIds.join(', ');
                } else {
                    document.getElementById('edit-card-links').value = '';
                }
            } catch (e) {
                document.getElementById('edit-card-links').value = '';
            }
            
            this.showModal('edit-card-modal');
        } catch (error) {
            this.showError('Failed to load card for editing');
        }
    }

    async handleEditCard(e) {
        e.preventDefault();
        
        const cardId = document.getElementById('edit-card-id').value;
        const zettelId = document.getElementById('edit-card-zettel-id').value.trim();
        const content = document.getElementById('edit-card-content').value;
        const linksText = document.getElementById('edit-card-links').value;

        const updateData = {
            zettel_id: zettelId || null,
            content: content || null,
            zettel_links: linksText ? linksText.split(',').map(l => l.trim()).filter(l => l) : null
        };

        try {
            await this.apiCall(`/cards/${cardId}`, {
                method: 'PUT',
                body: JSON.stringify(updateData)
            });

            // Reset form and close modal
            e.target.reset();
            this.closeModal(document.getElementById('edit-card-modal'));
            
            // Show success message
            this.showSuccess('Card updated successfully!');
            
            // Reload cards with current search
            if (this.currentSearchQuery) {
                await this.performSearch(this.currentSearchQuery);
            } else {
                await this.loadCards();
            }
        } catch (error) {
            this.showError('Failed to update card');
        }
    }

    async deleteCard(cardId) {
        if (!confirm('Are you sure you want to delete this card? This action cannot be undone.')) {
            return;
        }

        try {
            await this.apiCall(`/cards/${cardId}`, {
                method: 'DELETE'
            });

            // Show success message
            this.showSuccess('Card deleted successfully!');

            // Reload cards with current search
            if (this.currentSearchQuery) {
                await this.performSearch(this.currentSearchQuery);
            } else {
                await this.loadCards();
            }
        } catch (error) {
            this.showError('Failed to delete card');
        }
    }

    // Helper function to render LaTeX with MathJax
    renderMath(element) {
        if (window.MathJax) {
            MathJax.typesetPromise([element]).catch((err) => console.log(err.message));
        }
    }

    // Keyboard shortcuts handler
    handleKeyboardShortcuts(event) {
        // Don't handle shortcuts when typing in input fields
        if (event.target.tagName === 'INPUT' || event.target.tagName === 'TEXTAREA') {
            return;
        }

        const key = event.key;
        const isReviewMode = this.currentView === 'review' && this.currentQuiz;

        switch (key) {
            case '?':
                event.preventDefault();
                this.showKeyboardHelp();
                break;
            
            case 'Escape':
                event.preventDefault();
                this.handleEscapeKey();
                break;
            
            case ' ':
            case 'Spacebar':
                if (isReviewMode) {
                    event.preventDefault();
                    this.handleSpacebarInReview();
                }
                break;
            
            case '1':
                if (isReviewMode && this.isInRatingMode()) {
                    event.preventDefault();
                    this.rateCard(1);
                }
                break;
            
            case '2':
                if (isReviewMode && this.isInRatingMode()) {
                    event.preventDefault();
                    this.rateCard(2);
                }
                break;
            
            case '3':
                if (isReviewMode && this.isInRatingMode()) {
                    event.preventDefault();
                    this.rateCard(3);
                }
                break;
            
            case '4':
                if (isReviewMode && this.isInRatingMode()) {
                    event.preventDefault();
                    this.rateCard(4);
                }
                break;
        }
    }

    showKeyboardHelp() {
        this.showModal('keyboard-help-modal');
        
        // Re-initialize Feather icons for the modal
        if (window.feather) {
            feather.replace();
        }
    }

    handleEscapeKey() {
        // Close any open modal
        const openModals = document.querySelectorAll('.modal.active');
        if (openModals.length > 0) {
            openModals.forEach(modal => this.closeModal(modal));
        }
    }

    handleSpacebarInReview() {
        // Check if we're in question mode (submit button available)
        const submitButton = document.querySelector('.primary-btn:not(.rating-btn)');
        if (submitButton && submitButton.textContent.includes('Submit')) {
            this.submitAnswer();
        }
    }

    isInRatingMode() {
        // Check if rating buttons are currently visible
        const ratingButtons = document.querySelector('.rating-buttons');
        return ratingButtons && ratingButtons.style.display !== 'none';
    }

    // Progress indicator methods
    updateProgressIndicators() {
        if (!this.currentQuiz) return;
        
        const { questions, currentQuestion } = this.currentQuiz;
        const totalQuestions = questions.length;
        const currentQuestionNumber = currentQuestion + 1;
        const currentCardNumber = this.reviewSession.currentCardIndex + 1;
        const totalCards = this.reviewSession.totalCards;
        
        // Update question progress
        document.getElementById('progress-text').textContent = `Question ${currentQuestionNumber} of ${totalQuestions}`;
        document.getElementById('card-progress').textContent = `Card ${currentCardNumber} of ${totalCards}`;
        
        // Update progress bar
        const overallProgress = ((this.reviewSession.currentCardIndex * totalQuestions + currentQuestion) / (totalCards * totalQuestions)) * 100;
        document.getElementById('progress-fill').style.width = `${overallProgress}%`;
    }

    updateRemainingCount() {
        const remaining = this.reviewSession.totalCards - this.reviewSession.currentCardIndex;
        document.getElementById('remaining-number').textContent = remaining;
        
        if (remaining === 0) {
            document.getElementById('remaining-count').style.display = 'none';
        } else {
            document.getElementById('remaining-count').style.display = 'block';
        }
    }

    showCompletionScreen() {
        // Calculate session statistics
        const cardsReviewed = this.reviewSession.totalCards;
        const questionsAnswered = this.reviewSession.totalQuestions;
        const correctPercentage = questionsAnswered > 0 
            ? Math.round((this.reviewSession.correctAnswers / questionsAnswered) * 100)
            : 0;
        
        // Update statistics display
        document.getElementById('cards-reviewed').textContent = cardsReviewed;
        document.getElementById('questions-answered').textContent = questionsAnswered;
        document.getElementById('correct-percentage').textContent = `${correctPercentage}%`;
        
        // Show completion screen
        document.getElementById('completion-screen').style.display = 'flex';
        
        // Hide other elements
        document.getElementById('quiz-container').style.display = 'none';
        document.getElementById('review-progress-bar').style.display = 'none';
    }

    async startNewReview() {
        // Hide completion screen
        document.getElementById('completion-screen').style.display = 'none';
        
        // Reset animations and transitions
        document.getElementById('card-content-display').classList.remove('quiz-transition', 'fade-in');
        document.getElementById('quiz-questions').classList.remove('quiz-transition', 'fade-in');
        
        // Load new review session
        await this.loadReviewSession();
    }
}

// Initialize the application
const app = new LearningSystem();