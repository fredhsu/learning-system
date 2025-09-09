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
        this.cardCache = new Map(); // Cache for link preview data
        this.previewTimeouts = new Map(); // Track delayed loading states
        
        // Phase 2: Parallel processing configuration
        this.parallelProcessingMode = 'auto'; // 'auto', 'parallel', 'batch', 'sequential'
        this.maxConcurrentTasks = 5; // Default concurrency limit
        this.processingMetrics = null; // Store latest metrics
        
        this.init();
    }

    processWikiLinks(content) {
        // Process wiki-style links [[Link Text]] into styled HTML with preview functionality
        return content.replace(/\[\[([^\]]+)\]\]/g, (match, linkText) => {
            const trimmedText = linkText.trim();
            const uniqueId = `link-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
            
            return `<a href="#" class="wiki-link" id="${uniqueId}" 
                       onclick="app.handleWikiLinkClick('${trimmedText}'); return false;"
                       onmouseenter="app.showLinkPreview('${uniqueId}', '${trimmedText}')"
                       onmouseleave="app.hideLinkPreview('${uniqueId}')"
                       ontouchstart="app.handleLinkTouch('${uniqueId}', '${trimmedText}')"
                       ontouchend="app.handleLinkTouchEnd('${uniqueId}')">
                <i data-feather="link" class="wiki-link-icon"></i>
                <span>${trimmedText}</span>
            </a>`;
        });
    }

    renderMarkdown(content) {
        // First process wiki links, then markdown
        const processedContent = this.processWikiLinks(content);
        if (typeof marked !== 'undefined') {
            return marked.parse(processedContent);
        }
        return processedContent.replace(/\n/g, '<br>');
    }

    createPreviewContent(content) {
        // Process wiki links first, then create preview
        const linkedContent = this.processWikiLinks(content);
        const plainTextContent = linkedContent.replace(/[#*_`~]/g, '').substring(0, 100);
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

    toggleReviewCardContent() {
        const contentElement = document.getElementById('review-card-content');
        const indicator = document.getElementById('expand-indicator');
        const title = document.getElementById('review-card-title');
        
        if (!contentElement || !indicator) return;
        
        const isExpanded = contentElement.style.display !== 'none';
        
        if (isExpanded) {
            // Collapse content
            contentElement.style.display = 'none';
            indicator.textContent = '▼';
            title.classList.remove('expanded');
        } else {
            // Expand content
            contentElement.style.display = 'block';
            indicator.textContent = '▲';
            title.classList.add('expanded');
            
            // Re-render MathJax if available
            if (window.MathJax) {
                MathJax.typesetPromise([contentElement]).catch((err) => console.log(err.message));
            }
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
        
        // Populate cache with loaded cards for instant link previews
        cards.forEach(card => {
            if (card.title) {
                const cacheKey = card.title.toLowerCase();
                this.cardCache.set(cacheKey, card);
            }
        });
        
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
                        ${card.title ? `<h4 class="card-title">${card.title}</h4>` : ''}
                        <div class="card-header-meta">
                            <div class="card-header-meta-item">
                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                    <rect x="3" y="4" width="18" height="18" rx="2" ry="2"/>
                                    <line x1="16" y1="2" x2="16" y2="6"/>
                                    <line x1="8" y1="2" x2="8" y2="6"/>
                                    <line x1="3" y1="10" x2="21" y2="10"/>
                                </svg>
                                <span class="card-header-meta-value">${new Date(card.creation_date).toLocaleDateString()}</span>
                            </div>
                            <div class="card-header-meta-item">
                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                    <path d="M9 11H5a2 2 0 0 0-2 2v5a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-5a2 2 0 0 0-2-2h-4"/>
                                    <path d="M9 7l3-3 3 3"/>
                                    <path d="M12 4v8"/>
                                </svg>
                                <span class="card-header-meta-value">${card.reps} review${card.reps !== 1 ? 's' : ''}</span>
                            </div>
                            <div class="card-header-meta-item">
                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                    <circle cx="12" cy="12" r="3"/>
                                    <path d="M12 1v6m0 6v6m11-7h-6m-6 0H1"/>
                                </svg>
                                <span class="card-header-meta-value">${card.state}</span>
                            </div>
                            ${card.next_review ? `
                                <div class="card-header-meta-item">
                                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                        <circle cx="12" cy="12" r="10"/>
                                        <polyline points="12,6 12,12 16,14"/>
                                    </svg>
                                    <span class="card-header-meta-value">Next: ${new Date(card.next_review).toLocaleDateString()}</span>
                                </div>
                            ` : ''}
                        </div>
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
            const isLinkedCardsCollapsed = this.getLinkSectionPreference('linked-cards');
            const linkedCardsIcon = isLinkedCardsCollapsed ? '▶' : '▼';
            const linkedCardsHeaderClass = isLinkedCardsCollapsed ? 'linked-cards-header collapsed' : 'linked-cards-header';
            const linkedCardsListClass = isLinkedCardsCollapsed ? 'linked-cards-list collapsed' : 'linked-cards-list';
            
            linksHtml += `
                <div class="linked-cards">
                    <div class="${linkedCardsHeaderClass}" onclick="app.toggleLinkSection('linked-cards', '${cardId}')">
                        <div class="section-header-left">
                            <i data-feather="link"></i>
                            <span>Linked Cards</span>
                            <div class="section-count-badge">${linkedCards.length}</div>
                        </div>
                        <span class="section-toggle-icon">${linkedCardsIcon}</span>
                    </div>
                    <div class="${linkedCardsListClass}" id="linked-cards-list-${cardId}">
                        ${linkedCards.map(linkedCard => `
                            <a href="#" class="linked-card-item" onclick="app.navigateToCard('${linkedCard.id}'); return false;">
                                <span class="linked-card-zettel">${linkedCard.zettel_id}</span>
                                <span class="linked-card-preview">${linkedCard.title || this.truncateText(linkedCard.content, 60)}</span>
                            </a>
                        `).join('')}
                    </div>
                </div>
            `;
        }

        // Render backlinks
        if (backlinks.length > 0) {
            const isBacklinksCollapsed = this.getLinkSectionPreference('backlinks');
            const backlinksIcon = isBacklinksCollapsed ? '▶' : '▼';
            const backlinksHeaderClass = isBacklinksCollapsed ? 'backlinks-header collapsed' : 'backlinks-header';
            const backlinksListClass = isBacklinksCollapsed ? 'backlinks-list collapsed' : 'backlinks-list';
            
            linksHtml += `
                <div class="backlinks">
                    <div class="${backlinksHeaderClass}" onclick="app.toggleLinkSection('backlinks', '${cardId}')">
                        <div class="section-header-left">
                            <i data-feather="corner-down-left"></i>
                            <span>Backlinks</span>
                            <div class="section-count-badge">${backlinks.length}</div>
                        </div>
                        <span class="section-toggle-icon">${backlinksIcon}</span>
                    </div>
                    <div class="${backlinksListClass}" id="backlinks-list-${cardId}">
                        ${backlinks.map(backlinkCard => `
                            <a href="#" class="backlink-item" onclick="app.navigateToCard('${backlinkCard.id}'); return false;">
                                <span class="backlink-zettel">${backlinkCard.zettel_id}</span>
                                <span class="backlink-preview">${backlinkCard.title || this.truncateText(backlinkCard.content, 60)}</span>
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

    toggleLinkSection(sectionType, cardId) {
        const header = document.querySelector(`[onclick*="toggleLinkSection('${sectionType}', '${cardId}')"]`);
        const list = document.getElementById(`${sectionType}-list-${cardId}`);
        const toggleIcon = header.querySelector('.section-toggle-icon');
        
        if (!header || !list || !toggleIcon) return;

        const isCollapsed = list.classList.contains('collapsed');
        const prefKey = `linkSection_${sectionType}_collapsed`;

        if (isCollapsed) {
            // Expand
            list.classList.remove('collapsed');
            header.classList.remove('collapsed');
            toggleIcon.textContent = '▼';
            localStorage.setItem(prefKey, 'false');
        } else {
            // Collapse
            list.classList.add('collapsed');
            header.classList.add('collapsed');
            toggleIcon.textContent = '▶';
            localStorage.setItem(prefKey, 'true');
        }
    }

    getLinkSectionPreference(sectionType) {
        const prefKey = `linkSection_${sectionType}_collapsed`;
        return localStorage.getItem(prefKey) === 'true';
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
        const title = document.getElementById('card-title').value.trim();
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
            title: title || null,
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
            // Clear any previous feedback before starting new quiz
            document.getElementById('quiz-feedback').innerHTML = '';
            
            // Display the card content with transition
            const cardDisplay = document.getElementById('card-content-display');
            cardDisplay.classList.add('quiz-transition');
            
            const cardTitle = card.title || `Card ${card.zettel_id}`;
            const cardMeta = `Next review: ${new Date(card.next_review).toLocaleDateString()}`;
            
            cardDisplay.innerHTML = `
                <div class="review-card-header">
                    <h3 class="review-card-title" id="review-card-title" onclick="app.toggleReviewCardContent()">
                        ${cardTitle}
                        <span class="expand-indicator" id="expand-indicator">▼</span>
                    </h3>
                    <p class="card-meta">${cardMeta}</p>
                </div>
                <div class="review-card-content" id="review-card-content" style="display: none;">
                    <div class="card-content">${this.renderMarkdown(card.content)}</div>
                </div>
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
        const { questions } = this.currentQuiz;
        
        const container = document.getElementById('quiz-questions');
        container.classList.add('quiz-transition');
        
        // Render all questions at once
        const questionsHTML = questions.map((question, questionIndex) => {
            let questionHTML = `
                <div class="question" data-question-index="${questionIndex}">
                    <h4>Question ${questionIndex + 1} of ${questions.length}</h4>
                    <p>${question.question}</p>
            `;

            if (question.question_type === 'multiple_choice' && question.options) {
                questionHTML += `
                    <div class="options" data-question-index="${questionIndex}">
                        ${question.options.map((option, index) => `
                            <div class="option" data-option="${String.fromCharCode(65 + index)}" data-option-text="${option}" data-question-index="${questionIndex}">
                                ${String.fromCharCode(65 + index)}. ${option}
                            </div>
                        `).join('')}
                    </div>
                `;
            } else {
                questionHTML += `
                    <textarea class="short-answer" data-question-index="${questionIndex}" placeholder="Enter your answer..."></textarea>
                `;
            }

            questionHTML += '</div>';
            return questionHTML;
        }).join('');
        
        // Add submit button for all questions
        const submitButtonHTML = `
            <div class="batch-submit-container">
                <button class="primary-btn" onclick="app.submitAllAnswers()">Submit All Answers</button>
            </div>
        `;
        
        container.innerHTML = questionsHTML + submitButtonHTML;

        // Update progress indicators
        this.updateProgressIndicators();

        // Trigger fade-in animation
        setTimeout(() => {
            container.classList.add('fade-in');
        }, 50);

        // Add click listeners for multiple choice options
        document.querySelectorAll('.option').forEach(option => {
            option.addEventListener('click', () => {
                const questionIndex = option.dataset.questionIndex;
                // Remove selection from other options in the same question
                document.querySelectorAll(`[data-question-index="${questionIndex}"].option`).forEach(o => o.classList.remove('selected'));
                option.classList.add('selected');
                // Add visual feedback to the question
                const questionElement = document.querySelector(`[data-question-index="${questionIndex}"].question`);
                questionElement.classList.add('answering');
                
                // Check if all questions are answered
                this.checkAllQuestionsAnswered();
            });
        });

        // Add typing listener for short answer fields
        document.querySelectorAll('.short-answer').forEach(field => {
            field.addEventListener('input', () => {
                const questionIndex = field.dataset.questionIndex;
                const questionElement = document.querySelector(`[data-question-index="${questionIndex}"].question`);
                if (field.value.trim()) {
                    questionElement.classList.add('answering');
                } else {
                    questionElement.classList.remove('answering');
                }
                
                // Check if all questions are answered
                this.checkAllQuestionsAnswered();
            });
        });
    }

    checkAllQuestionsAnswered() {
        const { questions } = this.currentQuiz;
        const submitButton = document.querySelector('.batch-submit-container .primary-btn');
        
        let allAnswered = true;
        
        for (let i = 0; i < questions.length; i++) {
            const question = questions[i];
            
            if (question.question_type === 'multiple_choice') {
                const selectedOption = document.querySelector(`[data-question-index="${i}"].option.selected`);
                if (!selectedOption) {
                    allAnswered = false;
                    break;
                }
            } else {
                const textField = document.querySelector(`[data-question-index="${i}"].short-answer`);
                if (!textField || !textField.value.trim()) {
                    allAnswered = false;
                    break;
                }
            }
        }
        
        // Enable/disable submit button based on completion
        if (submitButton) {
            submitButton.disabled = !allAnswered;
            if (allAnswered) {
                submitButton.classList.add('ready');
            } else {
                submitButton.classList.remove('ready');
            }
        }
    }

    async submitAllAnswers() {
        const { card, questions } = this.currentQuiz;
        
        // Collect all answers
        const answers = this.collectAllAnswers();
        
        if (!answers) {
            this.showError('Please answer all questions before submitting');
            return;
        }

        // Disable submit button to prevent double submission
        const submitButton = document.querySelector('.batch-submit-container .primary-btn');
        submitButton.disabled = true;
        submitButton.textContent = 'Processing...';

        try {
            // Try parallel processing first, with fallback chain
            const result = await this.submitAnswersWithProcessingMode(answers, card);
            
            console.log('Processing result:', result);
            console.log('Processing metrics:', result.metrics);

            // Store metrics for performance monitoring
            this.processingMetrics = result.metrics;

            // Show feedback for all questions
            try {
                this.showBatchFeedbackWithMetrics(result, questions);
            } catch (feedbackError) {
                console.error('Error in showBatchFeedback:', feedbackError);
                this.showError('Failed to display grading results');
            }
        } catch (error) {
            console.error('Failed to submit answers:', error);
            this.showError('Failed to submit answers');
            // Re-enable button on error
            submitButton.disabled = false;
            submitButton.textContent = 'Submit All Answers';
        }
    }

    // Collect all answers from the form
    collectAllAnswers() {
        const answers = [];
        const questions = this.currentQuiz.questions;
        let allAnswered = true;
        
        for (let i = 0; i < questions.length; i++) {
            const question = questions[i];
            let answer = null;
            
            if (question.question_type === 'multiple_choice') {
                const selectedOption = document.querySelector(`[data-question-index="${i}"].option.selected`);
                if (!selectedOption) {
                    allAnswered = false;
                    break;
                }
                answer = selectedOption.dataset.optionText;
            } else {
                const textField = document.querySelector(`[data-question-index="${i}"].short-answer`);
                if (!textField || !textField.value.trim()) {
                    allAnswered = false;
                    break;
                }
                answer = textField.value.trim();
            }
            
            answers.push(answer);
        }
        
        return allAnswered ? answers : null;
    }

    // Submit answers with automatic processing mode selection
    async submitAnswersWithProcessingMode(answers, card) {
        const startTime = performance.now();
        
        // Determine processing mode
        const processingMode = this.determineProcessingMode(answers.length);
        
        console.log(`Using processing mode: ${processingMode} for ${answers.length} answers`);
        
        // Try parallel processing first (Phase 2)
        if (processingMode === 'parallel' || processingMode === 'auto') {
            try {
                const parallelResult = await this.submitParallelAnswers(answers, card);
                const endTime = performance.now();
                
                console.log(`Parallel processing completed in ${endTime - startTime}ms`);
                return parallelResult;
            } catch (parallelError) {
                console.warn('Parallel processing failed, falling back to batch:', parallelError);
            }
        }
        
        // Fallback to batch processing (Phase 1)
        try {
            const batchResult = await this.submitBatchAnswers(answers, card);
            const endTime = performance.now();
            
            console.log(`Batch processing completed in ${endTime - startTime}ms`);
            return batchResult;
        } catch (batchError) {
            console.warn('Batch processing failed, falling back to sequential:', batchError);
            
            // Final fallback: sequential processing
            return await this.submitSequentialAnswers(answers, card);
        }
    }

    // Determine optimal processing mode based on question count and browser capabilities
    determineProcessingMode(questionCount) {
        if (this.parallelProcessingMode !== 'auto') {
            return this.parallelProcessingMode;
        }
        
        // Auto-select based on question count
        if (questionCount >= 3) {
            return 'parallel'; // Parallel processing beneficial for 3+ questions
        } else if (questionCount >= 2) {
            return 'batch'; // Batch processing for 2 questions
        } else {
            return 'sequential'; // Single question uses sequential
        }
    }

    // Phase 2: Submit answers using parallel processing
    async submitParallelAnswers(answers, card) {
        const parallelRequest = {
            answers: answers.map((answer, index) => ({
                question_index: index,
                answer: answer
            })),
            processing_mode: 'parallel',
            max_concurrent_tasks: this.maxConcurrentTasks
        };

        return await this.apiCall(`/review/session/${this.reviewSession.sessionId}/answers/${card.id}/parallel`, {
            method: 'POST',
            body: JSON.stringify(parallelRequest)
        });
    }

    // Phase 1: Submit answers using batch processing
    async submitBatchAnswers(answers, card) {
        const batchRequest = {
            answers: answers.map((answer, index) => ({
                question_index: index,
                answer: answer
            }))
        };

        return await this.apiCall(`/review/session/${this.reviewSession.sessionId}/answers/${card.id}/batch`, {
            method: 'POST',
            body: JSON.stringify(batchRequest)
        });
    }

    // Final fallback: Submit answers sequentially (legacy mode)
    async submitSequentialAnswers(answers, card) {
        const results = [];
        
        for (let i = 0; i < answers.length; i++) {
            try {
                const result = await this.apiCall(`/review/session/${this.reviewSession.sessionId}/answer/${card.id}`, {
                    method: 'POST',
                    body: JSON.stringify({
                        question_index: i,
                        answer: answers[i]
                    })
                });
                
                results.push({
                    question_id: (i + 1).toString(),
                    is_correct: result.data.is_correct,
                    feedback: result.data.feedback,
                    suggested_rating: result.data.suggested_rating
                });
            } catch (error) {
                console.error(`Sequential submission failed for question ${i}:`, error);
                results.push({
                    question_id: (i + 1).toString(),
                    is_correct: false,
                    feedback: "Failed to grade this answer due to technical issues.",
                    suggested_rating: 2
                });
            }
        }
        
        return {
            success: true,
            data: results,
            metrics: {
                processing_mode_used: 'sequential_fallback',
                fallback_reason: 'Both parallel and batch processing failed'
            }
        };
    }

    async submitAnswer() {
        // Legacy method - now redirects to batch submission
        this.submitAllAnswers();
    }

    showBatchFeedback(results, questions) {
        console.log('showBatchFeedback called with results:', results, 'questions:', questions);
        
        if (!results) {
            console.error('No results provided to showBatchFeedback');
            this.showError('No grading results received');
            return;
        }
        
        if (!questions) {
            console.error('No questions provided to showBatchFeedback');
            this.showError('No questions available for feedback');
            return;
        }
        
        const feedbackContainer = document.getElementById('quiz-feedback');
        
        // Update statistics for all questions
        let totalCorrect = 0;
        results.forEach((grading, index) => {
            this.reviewSession.totalQuestions++;
            if (grading.is_correct) {
                this.reviewSession.correctAnswers++;
                totalCorrect++;
            }
            
            const question = questions[index];
            
            // Add visual feedback to options if multiple choice
            if (question.question_type === 'multiple_choice') {
                const selectedOption = document.querySelector(`[data-question-index="${index}"].option.selected`);
                const questionOptions = document.querySelectorAll(`[data-question-index="${index}"].option`);
                
                // Find and highlight correct option
                questionOptions.forEach(option => {
                    if (option.dataset.optionText === question.correct_answer) {
                        option.classList.add('correct-flash');
                    }
                });
                
                // Highlight incorrect selection if wrong
                if (selectedOption && !grading.is_correct) {
                    selectedOption.classList.add('incorrect-flash');
                }
            } else {
                // For short answer questions, highlight the input based on correctness
                const textField = document.querySelector(`[data-question-index="${index}"].short-answer`);
                if (textField) {
                    if (grading.is_correct) {
                        textField.classList.add('correct-answer');
                    } else {
                        textField.classList.add('incorrect-answer');
                    }
                }
            }
        });
        
        // Calculate overall performance and suggested rating
        const correctPercentage = (totalCorrect / questions.length) * 100;
        let suggestedRating = 3; // Default to Good
        
        if (correctPercentage >= 90) {
            suggestedRating = 4; // Easy
        } else if (correctPercentage >= 70) {
            suggestedRating = 3; // Good
        } else if (correctPercentage >= 50) {
            suggestedRating = 2; // Hard
        } else {
            suggestedRating = 1; // Again
        }
        
        // Store individual ratings for averaging (fallback to suggested rating)
        this.currentQuiz.questionRatings = results.map(result => result.suggested_rating || suggestedRating);
        
        // Create rating name helper function
        const getRatingName = (rating) => {
            const ratingNames = { 1: 'Again', 2: 'Hard', 3: 'Good', 4: 'Easy' };
            return ratingNames[rating] || 'Unknown';
        };
        
        // Generate detailed feedback HTML
        const detailedFeedbackHTML = results.map((grading, index) => {
            const question = questions[index];
            return `
                <div class="question-feedback ${grading.is_correct ? 'correct' : 'incorrect'}">
                    <div class="question-feedback-header">
                        <h5>Question ${index + 1}</h5>
                        <span class="result-badge ${grading.is_correct ? 'correct' : 'incorrect'}">
                            ${grading.is_correct ? '✓ Correct' : '✗ Incorrect'}
                        </span>
                    </div>
                    <p class="feedback-text">${grading.feedback}</p>
                    ${question.correct_answer && !grading.is_correct ? `<p class="correct-answer-display"><strong>Correct answer:</strong> ${question.correct_answer}</p>` : ''}
                </div>
            `;
        }).join('');
        
        feedbackContainer.innerHTML = `
            <div class="batch-feedback feedback-transition">
                <div class="batch-summary">
                    <h4>Results Summary</h4>
                    <div class="score-display">
                        <span class="score-number">${totalCorrect}/${questions.length}</span>
                        <span class="score-percentage">(${Math.round(correctPercentage)}%)</span>
                    </div>
                    <p class="suggested-rating"><strong>Suggested rating:</strong> ${getRatingName(suggestedRating)} (${suggestedRating})</p>
                </div>
                
                <div class="detailed-feedback">
                    <h5>Question Details</h5>
                    ${detailedFeedbackHTML}
                </div>
                
                <div class="rating-buttons">
                    <button class="rating-btn again rating-btn-with-shortcut ${suggestedRating === 1 ? 'suggested' : ''}" data-shortcut="1" onclick="app.rateCard(1)">Again</button>
                    <button class="rating-btn hard rating-btn-with-shortcut ${suggestedRating === 2 ? 'suggested' : ''}" data-shortcut="2" onclick="app.rateCard(2)">Hard</button>
                    <button class="rating-btn good rating-btn-with-shortcut ${suggestedRating === 3 ? 'suggested' : ''}" data-shortcut="3" onclick="app.rateCard(3)">Good</button>
                    <button class="rating-btn easy rating-btn-with-shortcut ${suggestedRating === 4 ? 'suggested' : ''}" data-shortcut="4" onclick="app.rateCard(4)">Easy</button>
                </div>
            </div>
        `;
    }

    // Enhanced feedback display with performance metrics
    showBatchFeedbackWithMetrics(result, questions) {
        // Extract data from either parallel or batch response format
        const results = result.data || result;
        const metrics = result.metrics;
        
        // Call the existing feedback display
        this.showBatchFeedback(results, questions);
        
        // Add performance metrics if available
        if (metrics) {
            this.displayPerformanceMetrics(metrics);
        }
    }

    // Display performance metrics from parallel processing
    displayPerformanceMetrics(metrics) {
        const feedbackContainer = document.getElementById('quiz-feedback');
        
        // Create metrics display
        const metricsHTML = `
            <div class="performance-metrics">
                <h5>Performance Metrics</h5>
                <div class="metrics-grid">
                    <div class="metric">
                        <label>Processing Mode:</label>
                        <span class="metric-value mode-${metrics.processing_mode_used}">${this.formatProcessingMode(metrics.processing_mode_used)}</span>
                    </div>
                    <div class="metric">
                        <label>Total Time:</label>
                        <span class="metric-value">${metrics.total_processing_time_ms}ms</span>
                    </div>
                    ${metrics.parallel_tasks_spawned > 0 ? `
                        <div class="metric">
                            <label>Concurrent Tasks:</label>
                            <span class="metric-value">${metrics.parallel_tasks_spawned}</span>
                        </div>
                        <div class="metric">
                            <label>Avg Task Time:</label>
                            <span class="metric-value">${metrics.average_task_duration_ms}ms</span>
                        </div>
                    ` : ''}
                    ${metrics.fallback_reason ? `
                        <div class="metric fallback-reason">
                            <label>Note:</label>
                            <span class="metric-value">${metrics.fallback_reason}</span>
                        </div>
                    ` : ''}
                </div>
                ${this.shouldShowPerformanceComparison(metrics) ? this.createPerformanceComparison(metrics) : ''}
            </div>
        `;
        
        // Insert metrics before rating buttons
        const ratingButtons = feedbackContainer.querySelector('.rating-buttons');
        if (ratingButtons) {
            ratingButtons.insertAdjacentHTML('beforebegin', metricsHTML);
        }
    }

    // Format processing mode for display
    formatProcessingMode(mode) {
        const modeNames = {
            'parallel': 'Parallel Processing',
            'batch_fallback': 'Batch Processing (Fallback)',
            'sequential_fallback': 'Sequential Processing (Fallback)',
            'batch': 'Batch Processing',
            'sequential': 'Sequential Processing'
        };
        return modeNames[mode] || mode;
    }

    // Check if we should show performance comparison
    shouldShowPerformanceComparison(metrics) {
        return metrics.processing_mode_used === 'parallel' && metrics.parallel_tasks_spawned >= 3;
    }

    // Create performance comparison visualization
    createPerformanceComparison(metrics) {
        const estimatedSequentialTime = metrics.parallel_tasks_spawned * metrics.average_task_duration_ms;
        const actualTime = metrics.total_processing_time_ms;
        const improvement = ((estimatedSequentialTime - actualTime) / estimatedSequentialTime * 100).toFixed(1);
        
        return `
            <div class="performance-comparison">
                <h6>Performance Impact</h6>
                <div class="comparison-bar">
                    <div class="bar-segment sequential" style="width: 100%">
                        <span>Estimated Sequential: ${estimatedSequentialTime}ms</span>
                    </div>
                    <div class="bar-segment parallel" style="width: ${(actualTime / estimatedSequentialTime * 100).toFixed(1)}%">
                        <span>Actual Parallel: ${actualTime}ms</span>
                    </div>
                </div>
                <p class="improvement-text">
                    <strong>${improvement}% faster</strong> than sequential processing
                </p>
            </div>
        `;
    }

    // Settings methods for parallel processing configuration
    setProcessingMode(mode) {
        this.parallelProcessingMode = mode;
        console.log(`Processing mode set to: ${mode}`);
    }

    setConcurrencyLimit(limit) {
        this.maxConcurrentTasks = Math.max(1, Math.min(10, limit));
        console.log(`Concurrency limit set to: ${this.maxConcurrentTasks}`);
    }

    // Get current processing configuration
    getProcessingConfig() {
        return {
            processingMode: this.parallelProcessingMode,
            maxConcurrentTasks: this.maxConcurrentTasks,
            lastMetrics: this.processingMetrics
        };
    }

    showFeedback(grading, question) {
        // Legacy method - maintained for backward compatibility
        this.showBatchFeedback([grading], [question]);
    }

    async rateCard(rating) {
        const { card, questions } = this.currentQuiz;
        
        // In batch mode, we use the stored questionRatings or default to the user's override
        const finalRating = rating; // User can override the suggested rating
        
        console.log(`Card rated with final rating: ${finalRating}`);

        try {
            // Submit the final rating to update FSRS
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
            document.getElementById('edit-card-title').value = card.title || '';
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
        const title = document.getElementById('edit-card-title').value.trim();
        const content = document.getElementById('edit-card-content').value;
        const linksText = document.getElementById('edit-card-links').value;

        const updateData = {
            zettel_id: zettelId || null,
            title: title || null,
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
        
        const { questions } = this.currentQuiz;
        const totalQuestions = questions.length;
        const currentCardNumber = this.reviewSession.currentCardIndex + 1;
        const totalCards = this.reviewSession.totalCards;
        
        // Update question progress - show all questions for current card
        document.getElementById('progress-text').textContent = `All ${totalQuestions} question${totalQuestions !== 1 ? 's' : ''}`;
        document.getElementById('card-progress').textContent = `Card ${currentCardNumber} of ${totalCards}`;
        
        // Update progress bar based on cards completed
        const overallProgress = (this.reviewSession.currentCardIndex / totalCards) * 100;
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

    async handleWikiLinkClick(linkText) {
        // First try to find card by title that matches the link text
        const cards = await this.apiCall('/cards').catch(() => []);
        
        // Look for exact title match first
        let targetCard = cards.find(card => 
            card.title && card.title.toLowerCase() === linkText.toLowerCase()
        );
        
        // If no title match, try to find by content containing the link text
        if (!targetCard) {
            targetCard = cards.find(card => 
                card.content.toLowerCase().includes(linkText.toLowerCase())
            );
        }
        
        // If found, navigate to the card
        if (targetCard) {
            this.navigateToCard(targetCard.id);
        } else {
            // Show toast notification for missing link target
            this.showError(`Card "${linkText}" not found`);
        }
    }

    async showLinkPreview(linkId, linkText) {
        const linkElement = document.getElementById(linkId);
        if (!linkElement) return;

        // Remove any existing preview and timeouts
        this.hideLinkPreview(linkId);

        // Create preview element
        const previewElement = document.createElement('div');
        previewElement.className = 'link-preview';
        previewElement.id = `preview-${linkId}`;
        
        // Position the preview (but keep it invisible initially)
        document.body.appendChild(previewElement);
        this.positionLinkPreview(linkElement, previewElement);

        // Set up delayed loading state (only if request takes too long)
        const loadingTimeout = setTimeout(() => {
            if (!previewElement.querySelector('.link-preview-header')) {
                previewElement.innerHTML = `
                    <div class="link-preview-loading">
                        <i data-feather="loader" style="width: 16px; height: 16px; animation: spin 1s linear infinite;"></i>
                        <span>Loading...</span>
                    </div>
                `;
                if (typeof feather !== 'undefined') {
                    feather.replace();
                }
                previewElement.classList.add('visible');
            }
        }, 200); // Only show loading after 200ms

        this.previewTimeouts.set(linkId, loadingTimeout);

        try {
            let targetCard;

            // Check cache first
            const cacheKey = linkText.toLowerCase();
            if (this.cardCache.has(cacheKey)) {
                targetCard = this.cardCache.get(cacheKey);
            } else {
                // Try to find in already loaded cards first (from this.allCards)
                if (this.allCards && this.allCards.length > 0) {
                    targetCard = this.allCards.find(card => 
                        card.title && card.title.toLowerCase() === cacheKey
                    );
                    
                    if (!targetCard) {
                        targetCard = this.allCards.find(card => 
                            card.content.toLowerCase().includes(cacheKey)
                        );
                    }
                }

                // If not found in loaded cards, fetch from API
                if (!targetCard) {
                    const cards = await this.apiCall('/cards');
                    targetCard = cards.find(card => 
                        card.title && card.title.toLowerCase() === cacheKey
                    );
                    
                    if (!targetCard) {
                        targetCard = cards.find(card => 
                            card.content.toLowerCase().includes(cacheKey)
                        );
                    }
                }

                // Cache the result (even if not found, to avoid repeated lookups)
                this.cardCache.set(cacheKey, targetCard || null);
            }

            // Clear the loading timeout since we have data
            clearTimeout(loadingTimeout);
            this.previewTimeouts.delete(linkId);

            if (targetCard) {
                // Show card preview
                this.renderLinkPreview(previewElement, targetCard);
            } else {
                // Show not found message
                previewElement.innerHTML = `
                    <div class="link-preview-error">
                        <i data-feather="alert-circle" class="link-preview-error-icon"></i>
                        <span>Card "${linkText}" not found</span>
                    </div>
                `;
            }

            // Re-initialize feather icons
            if (typeof feather !== 'undefined') {
                feather.replace();
            }

            // Show the preview with animation
            previewElement.classList.add('visible');

        } catch (error) {
            // Clear loading timeout on error
            clearTimeout(loadingTimeout);
            this.previewTimeouts.delete(linkId);

            // Show error message
            previewElement.innerHTML = `
                <div class="link-preview-error">
                    <i data-feather="alert-triangle" class="link-preview-error-icon"></i>
                    <span>Failed to load preview</span>
                </div>
            `;

            if (typeof feather !== 'undefined') {
                feather.replace();
            }

            previewElement.classList.add('visible');
        }
    }

    hideLinkPreview(linkId) {
        // Clear any pending loading timeout
        const loadingTimeout = this.previewTimeouts.get(linkId);
        if (loadingTimeout) {
            clearTimeout(loadingTimeout);
            this.previewTimeouts.delete(linkId);
        }

        const previewElement = document.getElementById(`preview-${linkId}`);
        if (previewElement) {
            previewElement.classList.remove('visible');
            setTimeout(() => {
                if (previewElement.parentNode) {
                    previewElement.parentNode.removeChild(previewElement);
                }
            }, 200); // Match the CSS transition duration
        }
    }

    positionLinkPreview(linkElement, previewElement) {
        const linkRect = linkElement.getBoundingClientRect();
        const viewportWidth = window.innerWidth;
        const viewportHeight = window.innerHeight;
        
        // Start positioning below the link
        let top = linkRect.bottom + window.scrollY + 8;
        let left = linkRect.left + window.scrollX;
        
        // Adjust if preview would go off screen horizontally
        const previewWidth = 350; // max-width from CSS
        if (left + previewWidth > viewportWidth - 20) {
            left = viewportWidth - previewWidth - 20;
        }
        if (left < 20) {
            left = 20;
        }
        
        // Adjust if preview would go off screen vertically
        const estimatedPreviewHeight = 200; // estimated height
        if (top + estimatedPreviewHeight > viewportHeight + window.scrollY - 20) {
            // Position above the link instead
            top = linkRect.top + window.scrollY - estimatedPreviewHeight - 8;
        }
        
        previewElement.style.left = `${left}px`;
        previewElement.style.top = `${top}px`;
    }

    renderLinkPreview(previewElement, card) {
        const zettelId = card.zettel_id;
        const title = card.title || 'Untitled';
        const content = this.createPreviewContent(card.content.substring(0, 150));
        const createdDate = new Date(card.creation_date).toLocaleDateString();
        
        previewElement.innerHTML = `
            <div class="link-preview-header">
                <h3 class="link-preview-title">${this.escapeHtml(title)}</h3>
                <span class="link-preview-zettel-id">${zettelId}</span>
            </div>
            <div class="link-preview-content">${content}</div>
            <div class="link-preview-metadata">
                <div class="link-preview-meta-item">
                    <i data-feather="calendar" class="link-preview-meta-icon"></i>
                    <span>${createdDate}</span>
                </div>
                <div class="link-preview-meta-item">
                    <i data-feather="eye" class="link-preview-meta-icon"></i>
                    <span>${card.reps || 0} reviews</span>
                </div>
            </div>
        `;
        
        // Add MathJax rendering if available
        if (window.MathJax) {
            MathJax.typesetPromise([previewElement]).catch((err) => console.log(err.message));
        }
    }

    truncateContent(content, maxLength) {
        if (!content) return '';
        
        // Remove markdown and wiki links for clean preview
        const cleanContent = content
            .replace(/\[\[([^\]]+)\]\]/g, '$1') // Remove wiki link syntax
            .replace(/[#*_`~]/g, '') // Remove basic markdown
            .replace(/\n+/g, ' ') // Replace newlines with spaces
            .trim();
            
        if (cleanContent.length <= maxLength) {
            return cleanContent;
        }
        
        // Find last complete word within limit
        const truncated = cleanContent.substring(0, maxLength);
        const lastSpace = truncated.lastIndexOf(' ');
        
        if (lastSpace > maxLength * 0.8) {
            return truncated.substring(0, lastSpace) + '...';
        }
        
        return truncated + '...';
    }

    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    handleLinkTouch(linkId, linkText) {
        // On touch devices, show preview after a brief delay
        this.touchPreviewTimeout = setTimeout(() => {
            this.showLinkPreview(linkId, linkText);
        }, 500); // 500ms delay for touch preview
    }

    handleLinkTouchEnd(linkId) {
        // Clear the touch preview timeout if touch ends before delay
        if (this.touchPreviewTimeout) {
            clearTimeout(this.touchPreviewTimeout);
            this.touchPreviewTimeout = null;
        }
        
        // Hide preview after a brief delay on touch end
        setTimeout(() => {
            this.hideLinkPreview(linkId);
        }, 2000); // Keep preview visible for 2 seconds on mobile
    }
}

// Initialize the application
const app = new LearningSystem();