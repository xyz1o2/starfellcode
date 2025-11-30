# 优先级 3 - 快速开始指南

## 🚀 5 分钟快速上手

### 1. 日志系统

```rust
use crate::core::logger::{Logger, LogLevel};

// 创建日志记录器
let logger = Logger::new(LogLevel::Debug);

// 记录日志
logger.trace("module", "detailed trace");
logger.debug("module", "debug info");
logger.info("module", "important info");
logger.warn("module", "warning");
logger.error("module", "error occurred");

// 查询日志
let all_logs = logger.get_entries();
let errors = logger.get_entries_by_level(LogLevel::Error);
let module_logs = logger.get_entries_by_module("module");

// 导出日志
let json_logs = logger.export_json();
let text_logs = logger.export_text();
```

### 2. 性能监控

```rust
use crate::core::performance_optimizer::PerformanceAnalyzer;
use std::time::Instant;

let analyzer = PerformanceAnalyzer::new();

// 记录操作性能
let start = Instant::now();
// ... 执行操作 ...
analyzer.record("operation_name", start.elapsed());

// 获取统计信息
if let Some(stats) = analyzer.get_stats("operation_name") {
    println!("Count: {}", stats.count);
    println!("Min: {:?}", stats.min);
    println!("Max: {:?}", stats.max);
    println!("Avg: {:?}", stats.avg);
    println!("Median: {:?}", stats.median);
}
```

### 3. 缓存优化

```rust
use crate::core::performance_optimizer::{SmartCache, CacheStrategy};

// 创建 LRU 缓存
let cache = SmartCache::new(CacheStrategy::LRU, 1000);

// 插入数据
cache.insert("key1", expensive_computation());

// 获取数据
if let Some(value) = cache.get(&"key1") {
    println!("Cache hit: {:?}", value);
}

// 删除数据
cache.remove(&"key1");

// 查询缓存大小
println!("Cache size: {}", cache.size());

// 清空缓存
cache.clear();
```

### 4. 批处理

```rust
use crate::core::performance_optimizer::BatchProcessor;
use std::time::Duration;

// 创建批处理器（批大小 100，超时 1 秒）
let processor = BatchProcessor::new(100, Duration::from_secs(1));

// 添加项
for item in items {
    if processor.add(item) {
        // 批处理已满，可以处理
        let batch = processor.flush();
        process_batch(batch);
    }
}

// 检查超时
if processor.should_flush() {
    let batch = processor.flush();
    process_batch(batch);
}
```

---

## 📋 常见用例

### 用例 1: 应用启动日志

```rust
let logger = Logger::new(LogLevel::Info);

logger.info("app", "Starting application");
logger.info("app", "Loading configuration");
logger.info("app", "Initializing database");
logger.info("app", "Application ready");

// 导出启动日志
println!("{}", logger.export_text());
```

### 用例 2: 性能分析

```rust
let analyzer = PerformanceAnalyzer::new();

// 分析 LLM 调用性能
let start = Instant::now();
let response = llm_client.generate(prompt).await?;
analyzer.record("llm_call", start.elapsed());

// 分析数据库查询性能
let start = Instant::now();
let results = db.query(sql).await?;
analyzer.record("db_query", start.elapsed());

// 获取性能报告
if let Some(stats) = analyzer.get_stats("llm_call") {
    println!("LLM 平均响应时间: {:?}", stats.avg);
}
```

### 用例 3: 缓存热数据

```rust
let cache = SmartCache::new(CacheStrategy::LRU, 10000);

// 缓存用户数据
cache.insert(format!("user_{}", user_id), user_data);

// 缓存代码分析结果
cache.insert(format!("code_{}", file_hash), analysis_result);

// 快速查询
if let Some(cached_user) = cache.get(&format!("user_{}", user_id)) {
    return Ok(cached_user);
}
```

### 用例 4: 批量处理消息

```rust
let processor = BatchProcessor::new(50, Duration::from_millis(500));

for message in incoming_messages {
    if processor.add(message) {
        let batch = processor.flush();
        send_batch_to_llm(batch).await?;
    }
}

// 处理剩余消息
if processor.should_flush() {
    let batch = processor.flush();
    send_batch_to_llm(batch).await?;
}
```

---

## 🧪 运行测试

```bash
# 运行所有优先级 3 测试
cargo test core::test_suite

# 运行特定测试
cargo test core::test_suite::test_logger_creation
cargo test core::test_suite::test_smart_cache_lru
cargo test core::test_suite::test_performance_analyzer

# 运行性能测试
cargo test core::test_suite::test_performance

# 运行并发测试
cargo test core::test_suite::test_concurrent

# 显示测试输出
cargo test core::test_suite -- --nocapture
```

---

## 🎯 集成检查清单

- [ ] 在 App 中添加 Logger 字段
- [ ] 在关键操作中添加日志记录
- [ ] 在 ConversationEngine 中集成 PerformanceAnalyzer
- [ ] 为热点操作启用 SmartCache
- [ ] 为批量操作启用 BatchProcessor
- [ ] 运行完整的测试套件
- [ ] 验证性能指标

---

## 📊 性能基准

| 操作 | 时间 | 备注 |
|------|------|------|
| 日志添加 (1000) | <100ms | 单线程 |
| 缓存操作 (1000) | <50ms | 单线程 |
| 日志查询 | <1ms | 平均 |
| 缓存查询 | <1ms | 平均 |
| 并发日志 (1000) | <200ms | 10 线程 |
| 并发缓存 (1000) | <100ms | 10 线程 |

---

## 💡 最佳实践

### 日志记录
✅ 使用适当的日志级别
✅ 包含足够的上下文
✅ 定期导出和分析
✅ 避免过度日志记录

### 缓存使用
✅ 为昂贵操作启用缓存
✅ 选择合适的缓存策略
✅ 监控缓存命中率
✅ 定期清理过期数据

### 性能监控
✅ 监控关键操作
✅ 设置性能目标
✅ 定期分析性能数据
✅ 及时优化瓶颈

---

## 🔧 故障排除

### 问题: 缓存大小不断增长

**解决方案**: 使用 TTL 策略或定期清理
```rust
let cache = SmartCache::new(CacheStrategy::TTL, 10000)
    .with_ttl(Duration::from_secs(3600));
```

### 问题: 日志文件过大

**解决方案**: 定期导出和清空日志
```rust
let json = logger.export_json();
save_to_file("logs.json", &json)?;
logger.clear();
```

### 问题: 性能监控开销大

**解决方案**: 只监控关键操作
```rust
// 只在生产环境监控
if cfg!(not(debug_assertions)) {
    analyzer.record("operation", duration);
}
```

---

## 📚 相关文档

- `PRIORITY_3_COMPLETE.md` - 完整实现文档
- `src/core/logger.rs` - Logger 源代码
- `src/core/performance_optimizer.rs` - 性能优化源代码
- `src/core/test_suite.rs` - 测试套件源代码

---

## 🚀 下一步

1. 集成日志系统到 App
2. 添加性能监控到关键路径
3. 为热点操作启用缓存
4. 运行完整的测试套件
5. 监控和优化性能

---

**快速开始完成！** 现在您可以开始使用优先级 3 的功能了。
