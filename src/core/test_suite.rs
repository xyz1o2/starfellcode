/// 完整的单元测试套件
/// 
/// 为所有核心模块提供全面的测试覆盖
/// 包括单元测试、集成测试和性能测试

#[cfg(test)]
mod comprehensive_tests {
    use crate::core::logger::*;
    use crate::core::performance_optimizer::*;
    use std::time::Duration;

    // ============ Logger 测试 ============

    #[test]
    fn test_logger_creation() {
        let logger = Logger::new(LogLevel::Info);
        logger.info("test", "test message");
        
        let entries = logger.get_entries();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_logger_filtering() {
        let logger = Logger::new(LogLevel::Info);
        logger.trace("test", "trace");
        logger.info("test", "info");
        logger.error("test", "error");

        let entries = logger.get_entries();
        assert_eq!(entries.len(), 2); // trace 被过滤
    }

    #[test]
    fn test_logger_by_level() {
        let logger = Logger::new(LogLevel::Trace);
        logger.info("test", "info");
        logger.error("test", "error");
        logger.warn("test", "warn");

        let errors = logger.get_entries_by_level(LogLevel::Error);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_logger_by_module() {
        let logger = Logger::new(LogLevel::Trace);
        logger.info("module1", "msg1");
        logger.info("module2", "msg2");
        logger.info("module1", "msg3");

        let module1 = logger.get_entries_by_module("module1");
        assert_eq!(module1.len(), 2);
    }

    #[test]
    fn test_log_entry_with_context() {
        let entry = LogEntry::new(LogLevel::Info, "test".to_string(), "message".to_string())
            .with_context("key".to_string(), "value".to_string());
        assert_eq!(entry.context.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_logger_export_json() {
        let logger = Logger::new(LogLevel::Trace);
        logger.info("test", "message");

        let json = logger.export_json();
        assert!(json.contains("\"level\":\"INFO\""));
        assert!(json.contains("\"message\":\"message\""));
    }

    // ============ SmartCache 测试 ============

    #[test]
    fn test_smart_cache_lru() {
        let cache = SmartCache::new(CacheStrategy::LRU, 2);
        cache.insert("a", 1);
        cache.insert("b", 2);
        cache.get(&"a"); // 访问 a
        cache.insert("c", 3); // 应该驱逐 b

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
        cache.insert("c", 3); // 应该驱逐 b

        assert_eq!(cache.get(&"a"), Some(1));
        assert_eq!(cache.get(&"b"), None);
    }

    #[test]
    fn test_smart_cache_fifo() {
        let cache = SmartCache::new(CacheStrategy::FIFO, 2);
        cache.insert("a", 1);
        cache.insert("b", 2);
        cache.insert("c", 3); // 应该驱逐 a

        assert_eq!(cache.get(&"a"), None);
        assert_eq!(cache.get(&"b"), Some(2));
        assert_eq!(cache.get(&"c"), Some(3));
    }

    // ============ BatchProcessor 测试 ============

    #[test]
    fn test_batch_processor() {
        let processor = BatchProcessor::new(3, Duration::from_secs(1));
        assert!(!processor.add(1));
        assert!(!processor.add(2));
        assert!(processor.add(3)); // 应该指示准备好刷新

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

    // ============ PerformanceAnalyzer 测试 ============

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

    #[test]
    fn test_performance_analyzer_median() {
        let analyzer = PerformanceAnalyzer::new();
        for i in 1..=5 {
            analyzer.record("op", Duration::from_millis(i * 10));
        }

        let stats = analyzer.get_stats("op").unwrap();
        assert_eq!(stats.median, Duration::from_millis(30));
    }

    // ============ 性能测试 ============

    #[test]
    fn test_performance_logger_add() {
        let logger = Logger::new(LogLevel::Trace);
        let start = std::time::Instant::now();

        for i in 0..1000 {
            logger.info("test", &format!("Message {}", i));
        }

        let duration = start.elapsed();
        // 应该在 100ms 以内
        assert!(duration.as_millis() < 100);
    }

    #[test]
    fn test_performance_cache_operations() {
        let cache = SmartCache::new(CacheStrategy::LRU, 1000);
        let start = std::time::Instant::now();

        for i in 0..1000 {
            cache.insert(format!("key_{}", i), i);
        }

        for i in 0..1000 {
            cache.get(&format!("key_{}", i));
        }

        let duration = start.elapsed();
        // 应该在 50ms 以内
        assert!(duration.as_millis() < 50);
    }

    // ============ 边界情况测试 ============

    #[test]
    fn test_edge_case_empty_logger() {
        let logger = Logger::new(LogLevel::Trace);
        let entries = logger.get_entries();
        assert_eq!(entries.len(), 0);
    }

    #[test]
    fn test_edge_case_large_logger() {
        let logger = Logger::new(LogLevel::Trace);
        
        for i in 0..1000 {
            logger.info("test", &format!("Message {}", i));
        }

        let entries = logger.get_entries();
        assert!(entries.len() > 0);
    }

    #[test]
    fn test_edge_case_cache_size_one() {
        let cache = SmartCache::new(CacheStrategy::LRU, 1);
        cache.insert("a", 1);
        cache.insert("b", 2);

        assert_eq!(cache.get(&"a"), None);
        assert_eq!(cache.get(&"b"), Some(2));
    }

    #[test]
    fn test_edge_case_batch_processor_zero_size() {
        let processor = BatchProcessor::new(0, Duration::from_secs(1));
        assert!(processor.add(1)); // 应该立即准备好
    }

    // ============ 并发测试 ============

    #[test]
    fn test_concurrent_logger() {
        use std::sync::Arc;
        use std::thread;

        let logger = Arc::new(Logger::new(LogLevel::Trace));
        let mut handles = vec![];

        for i in 0..10 {
            let logger_clone = Arc::clone(&logger);
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    logger_clone.info("test", &format!("Message from thread {} - {}", i, j));
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let entries = logger.get_entries();
        assert_eq!(entries.len(), 1000);
    }

    #[test]
    fn test_concurrent_cache() {
        use std::sync::Arc;
        use std::thread;

        let cache = Arc::new(SmartCache::new(CacheStrategy::LRU, 1000));
        let mut handles = vec![];

        for i in 0..10 {
            let cache_clone = Arc::clone(&cache);
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    cache_clone.insert(format!("key_{}_{}", i, j), i * 100 + j);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert!(cache.size() > 0);
    }
}
