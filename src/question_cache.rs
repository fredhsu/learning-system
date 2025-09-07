use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::models::{CachedQuestions, QuizQuestion, PreGenerationPriority};

/// In-memory question cache with TTL support and intelligent eviction
#[derive(Debug, Clone)]
pub struct QuestionCache {
    cache: Arc<RwLock<HashMap<Uuid, CachedQuestions>>>,
    max_size: usize,
    default_ttl_minutes: i64,
}

impl QuestionCache {
    pub fn new(max_size: usize, default_ttl_minutes: i64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_size,
            default_ttl_minutes,
        }
    }

    /// Store questions in cache with automatic expiration
    pub async fn cache_questions(&self, card_id: Uuid, questions: Vec<QuizQuestion>) {
        let now = Utc::now();
        let expires_at = now + Duration::minutes(self.default_ttl_minutes);
        
        let cached_questions = CachedQuestions {
            card_id,
            questions,
            generated_at: now,
            expires_at,
        };

        let mut cache = self.cache.write().await;
        
        // Clean up expired entries before adding new one
        self.cleanup_expired_entries(&mut cache, now).await;
        
        // If we're at capacity, remove oldest entry
        if cache.len() >= self.max_size {
            self.evict_oldest(&mut cache).await;
        }
        
        cache.insert(card_id, cached_questions);
        
        debug!("Cached questions for card {}, cache size: {}", card_id, cache.len());
    }

    /// Retrieve questions from cache if available and not expired
    pub async fn get_questions(&self, card_id: Uuid) -> Option<Vec<QuizQuestion>> {
        let mut cache = self.cache.write().await;
        let now = Utc::now();
        
        if let Some(cached) = cache.get(&card_id) {
            if cached.expires_at > now {
                debug!("Cache hit for card {}", card_id);
                return Some(cached.questions.clone());
            } else {
                debug!("Cache expired for card {}, removing", card_id);
                cache.remove(&card_id);
            }
        }
        
        debug!("Cache miss for card {}", card_id);
        None
    }

    /// Check if questions are cached and not expired
    pub async fn has_cached_questions(&self, card_id: Uuid) -> bool {
        let cache = self.cache.read().await;
        let now = Utc::now();
        
        if let Some(cached) = cache.get(&card_id) {
            cached.expires_at > now
        } else {
            false
        }
    }

    /// Get cache statistics for monitoring
    pub async fn get_stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let now = Utc::now();
        
        let total_entries = cache.len();
        let expired_entries = cache.values()
            .filter(|cached| cached.expires_at <= now)
            .count();
        
        CacheStats {
            total_entries,
            expired_entries,
            active_entries: total_entries - expired_entries,
            max_size: self.max_size,
            hit_rate_estimate: 0.0, // Could be tracked with additional counters
        }
    }

    /// Manual cache cleanup - removes expired entries
    pub async fn cleanup(&self) {
        let mut cache = self.cache.write().await;
        let now = Utc::now();
        self.cleanup_expired_entries(&mut cache, now).await;
    }

    /// Clear all cached questions
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        info!("Question cache cleared");
    }

    /// Private helper to remove expired entries
    async fn cleanup_expired_entries(&self, cache: &mut HashMap<Uuid, CachedQuestions>, now: DateTime<Utc>) {
        let expired_keys: Vec<Uuid> = cache.iter()
            .filter(|(_, cached)| cached.expires_at <= now)
            .map(|(key, _)| *key)
            .collect();
        
        for key in expired_keys {
            cache.remove(&key);
            debug!("Removed expired cache entry for card {}", key);
        }
    }

    /// Private helper to evict oldest entry when at capacity
    async fn evict_oldest(&self, cache: &mut HashMap<Uuid, CachedQuestions>) {
        if let Some(oldest_key) = cache.iter()
            .min_by_key(|(_, cached)| cached.generated_at)
            .map(|(key, _)| *key) {
            cache.remove(&oldest_key);
            debug!("Evicted oldest cache entry for card {}", oldest_key);
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub active_entries: usize,
    pub max_size: usize,
    pub hit_rate_estimate: f64,
}

/// Background pre-generation queue manager
#[derive(Debug, Clone)]
pub struct PreGenerationQueue {
    queue: Arc<RwLock<Vec<PreGenerationTask>>>,
    max_queue_size: usize,
}

#[derive(Debug, Clone)]
pub struct PreGenerationTask {
    pub card_id: Uuid,
    pub priority: PreGenerationPriority,
    pub queued_at: DateTime<Utc>,
}

impl PreGenerationQueue {
    pub fn new(max_queue_size: usize) -> Self {
        Self {
            queue: Arc::new(RwLock::new(Vec::new())),
            max_queue_size,
        }
    }

    /// Add card to pre-generation queue
    pub async fn enqueue(&self, card_id: Uuid, priority: PreGenerationPriority) {
        let mut queue = self.queue.write().await;
        
        // Check if already queued
        if queue.iter().any(|task| task.card_id == card_id) {
            debug!("Card {} already in pre-generation queue", card_id);
            return;
        }
        
        // If at capacity, remove lowest priority task
        if queue.len() >= self.max_queue_size {
            queue.sort_by_key(|task| match task.priority {
                PreGenerationPriority::Immediate => 0,
                PreGenerationPriority::NextCard => 1,
                PreGenerationPriority::Background => 2,
            });
            queue.pop(); // Remove lowest priority (Background)
            debug!("Pre-generation queue at capacity, removed background task");
        }
        
        let task = PreGenerationTask {
            card_id,
            priority: priority.clone(),
            queued_at: Utc::now(),
        };
        
        queue.push(task);
        debug!("Added card {} to pre-generation queue with priority {:?}", card_id, priority);
    }

    /// Get next task to process (highest priority first)
    pub async fn dequeue(&self) -> Option<PreGenerationTask> {
        let mut queue = self.queue.write().await;
        
        if queue.is_empty() {
            return None;
        }
        
        // Sort by priority (Immediate = 0, NextCard = 1, Background = 2)
        queue.sort_by_key(|task| match task.priority {
            PreGenerationPriority::Immediate => 0,
            PreGenerationPriority::NextCard => 1,
            PreGenerationPriority::Background => 2,
        });
        
        let task = queue.remove(0);
        debug!("Dequeued card {} from pre-generation queue", task.card_id);
        Some(task)
    }

    /// Get queue size for monitoring
    pub async fn size(&self) -> usize {
        let queue = self.queue.read().await;
        queue.len()
    }
}