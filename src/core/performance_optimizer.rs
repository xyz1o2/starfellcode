/// 性能优化模块
/// 
/// 提供性能优化工具和最佳实践
/// 包括缓存策略、批处理、异步优化等

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 缓存策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheStrategy {
    /// LRU - 最近最少使用
    LRU,
    /// LFU - 最不经常使用
    LFU,
    /// FIFO - 先进先出
    FIFO,
    /// TTL - 时间到期
    TTL,
}

/// 缓存条目
#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    created_at: Instant,
    last_accessed: Instant,
    access_count: u32,
}

/// 智能缓存
pub struct SmartCache<K, V> {
    data: Arc<Mutex<HashMap<K, CacheEntry<V>>>>,
    strategy: CacheStrategy,
    max_size: usize,
    ttl: Option<Duration>,
}

impl<K: std::hash::Hash + Eq + Clone, V: Clone> SmartCache<K, V> {
    pub fn new(strategy: CacheStrategy, max_size: usize) -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
            strategy,
            max_size,
            ttl: None,
        }
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }

    pub fn insert(&self, key: K, value: V) {
        if let Ok(mut data) = self.data.lock() {
            // 检查是否需要清理
            if data.len() >= self.max_size {
                self.evict(&mut data);
            }

            data.insert(
                key,
                CacheEntry {
                    value,
                    created_at: Instant::now(),
                    last_accessed: Instant::now(),
                    access_count: 0,
                },
            );
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        if let Ok(mut data) = self.data.lock() {
            if let Some(entry) = data.get_mut(key) {
                // 检查 TTL
                if let Some(ttl) = self.ttl {
                    if entry.created_at.elapsed() > ttl {
                        data.remove(key);
                        return None;
                    }
                }

                entry.last_accessed = Instant::now();
                entry.access_count += 1;
                return Some(entry.value.clone());
            }
        }
        None
    }

    pub fn remove(&self, key: &K) {
        if let Ok(mut data) = self.data.lock() {
            data.remove(key);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut data) = self.data.lock() {
            data.clear();
        }
    }

    pub fn size(&self) -> usize {
        self.data.lock().ok().map(|d| d.len()).unwrap_or(0)
    }

    fn evict(&self, data: &mut HashMap<K, CacheEntry<V>>) {
        match self.strategy {
            CacheStrategy::LRU => {
                if let Some(key) = data
                    .iter()
                    .min_by_key(|(_, entry)| entry.last_accessed)
                    .map(|(k, _)| k.clone())
                {
                    data.remove(&key);
                }
            }
            CacheStrategy::LFU => {
                if let Some(key) = data
                    .iter()
                    .min_by_key(|(_, entry)| entry.access_count)
                    .map(|(k, _)| k.clone())
                {
                    data.remove(&key);
                }
            }
            CacheStrategy::FIFO => {
                if let Some(key) = data
                    .iter()
                    .min_by_key(|(_, entry)| entry.created_at)
                    .map(|(k, _)| k.clone())
                {
                    data.remove(&key);
                }
            }
            CacheStrategy::TTL => {
                let now = Instant::now();
                if let Some(ttl) = self.ttl {
                    data.retain(|_, entry| now.duration_since(entry.created_at) <= ttl);
                }
            }
        }
    }
}

/// 批处理器
pub struct BatchProcessor<T> {
    batch: Arc<Mutex<Vec<T>>>,
    batch_size: usize,
    timeout: Duration,
    last_flush: Arc<Mutex<Instant>>,
}

impl<T: Clone> BatchProcessor<T> {
    pub fn new(batch_size: usize, timeout: Duration) -> Self {
        Self {
            batch: Arc::new(Mutex::new(Vec::new())),
            batch_size,
            timeout,
            last_flush: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn add(&self, item: T) -> bool {
        if let Ok(mut batch) = self.batch.lock() {
            batch.push(item);
            Ok(batch.len() >= self.batch_size)
        } else {
            Err(())
        }
        .unwrap_or(false)
    }

    pub fn should_flush(&self) -> bool {
        if let Ok(last_flush) = self.last_flush.lock() {
            last_flush.elapsed() > self.timeout
        } else {
            false
        }
    }

    pub fn flush(&self) -> Vec<T> {
        if let Ok(mut batch) = self.batch.lock() {
            if let Ok(mut last_flush) = self.last_flush.lock() {
                *last_flush = Instant::now();
            }
            batch.drain(..).collect()
        } else {
            Vec::new()
        }
    }

    pub fn size(&self) -> usize {
        self.batch.lock().ok().map(|b| b.len()).unwrap_or(0)
    }
}

/// 连接池
pub struct ConnectionPool<T: Clone> {
    available: Arc<Mutex<Vec<T>>>,
    in_use: Arc<Mutex<Vec<T>>>,
    factory: Arc<dyn Fn() -> T + Send + Sync>,
    max_size: usize,
}

impl<T: Clone + Send + Sync + 'static> ConnectionPool<T> {
    pub fn new<F>(factory: F, max_size: usize) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            available: Arc::new(Mutex::new(Vec::new())),
            in_use: Arc::new(Mutex::new(Vec::new())),
            factory: Arc::new(factory),
            max_size,
        }
    }

    pub fn acquire(&self) -> Option<T> {
        if let Ok(mut available) = self.available.lock() {
            if let Some(conn) = available.pop() {
                if let Ok(mut in_use) = self.in_use.lock() {
                    in_use.push(conn.clone());
                    return Some(conn);
                }
            }
        }

        // 创建新连接
        if let Ok(in_use) = self.in_use.lock() {
            if in_use.len() < self.max_size {
                let conn = (self.factory)();
                if let Ok(mut in_use_mut) = self.in_use.lock() {
                    in_use_mut.push(conn.clone());
                    return Some(conn);
                }
            }
        }

        None
    }

    pub fn release(&self, conn: T) {
        if let Ok(mut in_use) = self.in_use.lock() {
            in_use.retain(|c| c.clone() != conn);
        }

        if let Ok(mut available) = self.available.lock() {
            available.push(conn);
        }
    }

    pub fn size(&self) -> (usize, usize) {
        let available = self.available.lock().ok().map(|a| a.len()).unwrap_or(0);
        let in_use = self.in_use.lock().ok().map(|i| i.len()).unwrap_or(0);
        (available, in_use)
    }
}

/// 性能分析器
pub struct PerformanceAnalyzer {
    metrics: Arc<Mutex<HashMap<String, Vec<Duration>>>>,
}

impl PerformanceAnalyzer {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn record(&self, name: &str, duration: Duration) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.entry(name.to_string()).or_insert_with(Vec::new).push(duration);
        }
    }

    pub fn get_stats(&self, name: &str) -> Option<PerformanceStats> {
        if let Ok(metrics) = self.metrics.lock() {
            if let Some(durations) = metrics.get(name) {
                if durations.is_empty() {
                    return None;
                }

                let mut sorted = durations.clone();
                sorted.sort();

                let sum: Duration = sorted.iter().sum();
                let count = sorted.len();
                let avg = sum / count as u32;
                let min = sorted[0];
                let max = sorted[count - 1];
                let median = sorted[count / 2];

                return Some(PerformanceStats {
                    count,
                    min,
                    max,
                    avg,
                    median,
                });
            }
        }
        None
    }

    pub fn clear(&self) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.clear();
        }
    }
}

impl Default for PerformanceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// 性能统计
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub count: usize,
    pub min: Duration,
    pub max: Duration,
    pub avg: Duration,
    pub median: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_cache_lru() {
        let cache = SmartCache::new(CacheStrategy::LRU, 2);
        cache.insert("a", 1);
        cache.insert("b", 2);
        cache.get(&"a"); // Access a
        cache.insert("c", 3); // Should evict b

        assert_eq!(cache.get(&"a"), Some(1));
        assert_eq!(cache.get(&"b"), None);
        assert_eq!(cache.get(&"c"), Some(3));
    }

    #[test]
    fn test_smart_cache_lfu() {
        let cache = SmartCache::new(CacheStrategy::LFU, 2);
        cache.insert("a", 1);
        cache.insert("b", 2);
        cache.get(&"a");
        cache.get(&"a");
        cache.insert("c", 3); // Should evict b

        assert_eq!(cache.get(&"a"), Some(1));
        assert_eq!(cache.get(&"b"), None);
    }

    #[test]
    fn test_batch_processor() {
        let processor = BatchProcessor::new(3, Duration::from_secs(1));
        assert!(!processor.add(1));
        assert!(!processor.add(2));
        assert!(processor.add(3)); // Should indicate ready to flush

        let batch = processor.flush();
        assert_eq!(batch.len(), 3);
        assert_eq!(processor.size(), 0);
    }

    #[test]
    fn test_batch_processor_timeout() {
        let processor = BatchProcessor::new(10, Duration::from_millis(100));
        processor.add(1);
        processor.add(2);

        std::thread::sleep(Duration::from_millis(150));
        assert!(processor.should_flush());

        let batch = processor.flush();
        assert_eq!(batch.len(), 2);
    }

    #[test]
    fn test_performance_analyzer() {
        let analyzer = PerformanceAnalyzer::new();
        analyzer.record("op", Duration::from_millis(10));
        analyzer.record("op", Duration::from_millis(20));
        analyzer.record("op", Duration::from_millis(30));

        let stats = analyzer.get_stats("op").unwrap();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.min, Duration::from_millis(10));
        assert_eq!(stats.max, Duration::from_millis(30));
    }
}
