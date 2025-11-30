use std::time::Duration;

use tokio::time::sleep;

/// 控制重试行为的配置
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// 最大尝试次数（包含首次调用）
    pub max_attempts: u32,
    /// 初始退避延迟（毫秒）
    pub initial_delay_ms: u64,
    /// 每次重试的退避倍数
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 500,
            backoff_multiplier: 2.0,
        }
    }
}

/// 可判断是否允许重试的错误类型
pub trait RetryableError: std::error::Error {
    /// 当前错误是否可以通过重试恢复
    fn retryable(&self) -> bool;
}

/// 通用异步重试执行器
#[derive(Clone)]
pub struct RetryHandler {
    config: RetryConfig,
}

impl RetryHandler {
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &RetryConfig {
        &self.config
    }

    /// 执行带重试的异步操作
    pub async fn execute_with_retry<F, Fut, T, E>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut(u32) -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: RetryableError,
    {
        let mut delay = self.config.initial_delay_ms;
        let max_attempts = self.config.max_attempts.max(1);

        for attempt in 0..max_attempts {
            match operation(attempt).await {
                Ok(value) => return Ok(value),
                Err(err) => {
                    let should_retry = err.retryable() && attempt + 1 < max_attempts;
                    if !should_retry {
                        return Err(err);
                    }

                    sleep(Duration::from_millis(delay)).await;
                    delay = (delay as f64 * self.config.backoff_multiplier) as u64;
                }
            }
        }

        unreachable!("循环只能通过返回语句退出");
    }
}
