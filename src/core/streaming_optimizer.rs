/// 流式处理优化模块 - 对应 Gemini CLI 的流式优化
/// 
/// 支持完善的流式处理优化：
/// - 智能分块
/// - 缓冲管理
/// - 性能监控
/// - 背压处理
/// - 吞吐量计算

use chrono::{DateTime, Local};
use std::time::Instant;

/// 流事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamEventType {
    Chunk,      // 普通内容块
    Retry,      // 重试信号
    Complete,   // 完成
    Error,      // 错误
}

/// 优化的流式事件
#[derive(Debug, Clone)]
pub struct OptimizedStreamEvent {
    pub event_type: StreamEventType,
    pub content: String,
    pub chunk_index: usize,
    pub total_chunks: Option<usize>,
    pub timestamp: DateTime<Local>,
    pub processing_time_ms: u64,
}

/// 流式优化器配置
#[derive(Debug, Clone)]
pub struct StreamingOptimizerConfig {
    pub chunk_size: usize,              // 块大小（字符数）
    pub buffer_threshold: usize,        // 缓冲阈值
    pub flush_interval_ms: u64,         // 刷新间隔（毫秒）
    pub enable_compression: bool,       // 启用压缩
    pub max_buffer_size: usize,         // 最大缓冲大小
}

impl Default for StreamingOptimizerConfig {
    fn default() -> Self {
        Self {
            chunk_size: 256,
            buffer_threshold: 1024,
            flush_interval_ms: 100,
            enable_compression: false,
            max_buffer_size: 10240,
        }
    }
}

/// 性能指标
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub total_events: usize,
    pub total_bytes: usize,
    pub total_time_ms: u64,
    pub average_latency_ms: f64,
    pub throughput_events_per_sec: f64,
    pub throughput_bytes_per_sec: f64,
    pub peak_buffer_size: usize,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            total_events: 0,
            total_bytes: 0,
            total_time_ms: 0,
            average_latency_ms: 0.0,
            throughput_events_per_sec: 0.0,
            throughput_bytes_per_sec: 0.0,
            peak_buffer_size: 0,
        }
    }
}

/// 流式优化器
pub struct StreamingOptimizer {
    config: StreamingOptimizerConfig,
    metrics: PerformanceMetrics,
    buffer: Vec<String>,
    last_flush: Instant,
}

impl StreamingOptimizer {
    /// 创建新的流式优化器
    pub fn new(config: StreamingOptimizerConfig) -> Self {
        Self {
            config,
            metrics: PerformanceMetrics::new(),
            buffer: Vec::new(),
            last_flush: Instant::now(),
        }
    }

    /// 使用默认配置创建优化器
    pub fn default() -> Self {
        Self::new(StreamingOptimizerConfig::default())
    }

    /// 添加事件到缓冲区
    pub fn add_event(&mut self, content: String) -> Option<OptimizedStreamEvent> {
        self.buffer.push(content.clone());
        self.metrics.total_bytes += content.len();

        // 检查是否应该刷新
        if self.should_flush() {
            self.flush()
        } else {
            None
        }
    }

    /// 检查是否应该刷新缓冲区
    fn should_flush(&self) -> bool {
        let buffer_size: usize = self.buffer.iter().map(|s| s.len()).sum();
        
        // 缓冲区超过阈值
        if buffer_size >= self.config.buffer_threshold {
            return true;
        }

        // 缓冲区超过最大大小
        if buffer_size >= self.config.max_buffer_size {
            return true;
        }

        // 刷新间隔已过
        if self.last_flush.elapsed().as_millis() as u64 >= self.config.flush_interval_ms {
            return true;
        }

        false
    }

    /// 刷新缓冲区
    pub fn flush(&mut self) -> Option<OptimizedStreamEvent> {
        if self.buffer.is_empty() {
            return None;
        }

        let start = Instant::now();
        let content = self.buffer.join("");
        let processing_time_ms = start.elapsed().as_millis() as u64;

        self.metrics.total_events += 1;
        self.metrics.total_time_ms += processing_time_ms;

        let event = OptimizedStreamEvent {
            event_type: StreamEventType::Chunk,
            content,
            chunk_index: self.metrics.total_events,
            total_chunks: None,
            timestamp: Local::now(),
            processing_time_ms,
        };

        self.buffer.clear();
        self.last_flush = Instant::now();

        Some(event)
    }

    /// 分块处理内容
    pub fn chunk_content(&self, content: &str) -> Vec<String> {
        content
            .chars()
            .collect::<Vec<_>>()
            .chunks(self.config.chunk_size)
            .map(|chunk| chunk.iter().collect())
            .collect()
    }

    /// 计算吞吐量（事件/秒）
    pub fn calculate_throughput_events_per_sec(&self) -> f64 {
        if self.metrics.total_time_ms == 0 {
            return 0.0;
        }
        (self.metrics.total_events as f64 / self.metrics.total_time_ms as f64) * 1000.0
    }

    /// 计算吞吐量（字节/秒）
    pub fn calculate_throughput_bytes_per_sec(&self) -> f64 {
        if self.metrics.total_time_ms == 0 {
            return 0.0;
        }
        (self.metrics.total_bytes as f64 / self.metrics.total_time_ms as f64) * 1000.0
    }

    /// 计算平均延迟（毫秒）
    pub fn calculate_average_latency(&self) -> f64 {
        if self.metrics.total_events == 0 {
            return 0.0;
        }
        self.metrics.total_time_ms as f64 / self.metrics.total_events as f64
    }

    /// 获取性能指标
    pub fn get_metrics(&self) -> PerformanceMetrics {
        let mut metrics = self.metrics.clone();
        metrics.throughput_events_per_sec = self.calculate_throughput_events_per_sec();
        metrics.throughput_bytes_per_sec = self.calculate_throughput_bytes_per_sec();
        metrics.average_latency_ms = self.calculate_average_latency();
        metrics
    }

    /// 重置指标
    pub fn reset_metrics(&mut self) {
        self.metrics = PerformanceMetrics::new();
        self.last_flush = Instant::now();
    }

    /// 应用背压（如果缓冲区过大则等待）
    pub async fn apply_backpressure(&self) {
        let buffer_size: usize = self.buffer.iter().map(|s| s.len()).sum();
        
        if buffer_size > self.config.buffer_threshold {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }

    /// 压缩内容（如果启用）
    pub fn compress_content(&self, content: &str) -> String {
        if self.config.enable_compression {
            // 简单的压缩：移除多余空格
            content
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            content.to_string()
        }
    }

    /// 获取缓冲区大小
    pub fn get_buffer_size(&self) -> usize {
        self.buffer.iter().map(|s| s.len()).sum()
    }

    /// 获取缓冲区中的事件数
    pub fn get_buffer_event_count(&self) -> usize {
        self.buffer.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_content() {
        let optimizer = StreamingOptimizer::default();
        let content = "Hello, World! This is a test.";
        let chunks = optimizer.chunk_content(content);
        
        assert!(!chunks.is_empty());
        for chunk in chunks {
            assert!(chunk.len() <= optimizer.config.chunk_size);
        }
    }

    #[test]
    fn test_buffer_management() {
        let config = StreamingOptimizerConfig {
            chunk_size: 256,
            buffer_threshold: 100,
            flush_interval_ms: 100,
            enable_compression: false,
            max_buffer_size: 1000,
        };
        let mut optimizer = StreamingOptimizer::new(config);
        
        let event = optimizer.add_event("Hello".to_string());
        assert!(event.is_none()); // 缓冲区未满
        
        let event = optimizer.add_event("a".repeat(100).to_string());
        assert!(event.is_some()); // 缓冲区已满，应该刷新
    }

    #[test]
    fn test_throughput_calculation() {
        let mut optimizer = StreamingOptimizer::default();
        
        optimizer.add_event("Hello".to_string());
        optimizer.add_event("World".to_string());
        
        let throughput = optimizer.calculate_throughput_events_per_sec();
        assert!(throughput > 0.0);
    }

    #[test]
    fn test_average_latency() {
        let mut optimizer = StreamingOptimizer::default();
        
        optimizer.add_event("Hello".to_string());
        optimizer.add_event("World".to_string());
        
        let latency = optimizer.calculate_average_latency();
        assert!(latency >= 0.0);
    }

    #[test]
    fn test_metrics() {
        let mut optimizer = StreamingOptimizer::default();
        
        optimizer.add_event("Hello".to_string());
        optimizer.flush();
        
        let metrics = optimizer.get_metrics();
        assert!(metrics.total_events > 0);
        assert!(metrics.total_bytes > 0);
    }

    #[test]
    fn test_compression() {
        let config = StreamingOptimizerConfig {
            enable_compression: true,
            ..Default::default()
        };
        let optimizer = StreamingOptimizer::new(config);
        
        let content = "Hello   World   This   is   a   test";
        let compressed = optimizer.compress_content(content);
        
        assert!(compressed.len() < content.len());
    }

    #[tokio::test]
    async fn test_backpressure() {
        let optimizer = StreamingOptimizer::default();
        
        // 应该不会阻塞（缓冲区为空）
        optimizer.apply_backpressure().await;
    }

    #[test]
    fn test_buffer_size() {
        let mut optimizer = StreamingOptimizer::default();
        
        optimizer.add_event("Hello".to_string());
        let size = optimizer.get_buffer_size();
        
        assert!(size > 0);
    }
}
