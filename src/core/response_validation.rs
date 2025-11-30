use std::fmt::{Display, Formatter};

use crate::core::retry_handler::{RetryConfig, RetryHandler, RetryableError};

/// 响应验证错误
#[derive(Debug)]
pub enum ResponseError {
    Empty,
    HasErrorMarker,
    TooShort,
}

impl Display for ResponseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseError::Empty => write!(f, "response content is empty"),
            ResponseError::HasErrorMarker => write!(f, "response contains error markers"),
            ResponseError::TooShort => write!(f, "response is too short to be meaningful"),
        }
    }
}

impl std::error::Error for ResponseError {}

impl RetryableError for ResponseError {
    fn retryable(&self) -> bool {
        matches!(self, ResponseError::Empty | ResponseError::TooShort)
    }
}

/// 响应验证器
#[derive(Clone)]
pub struct ResponseValidator {
    retry: RetryHandler,
}

impl ResponseValidator {
    pub fn new(config: RetryConfig) -> Self {
        Self {
            retry: RetryHandler::new(config),
        }
    }

    pub fn retry_config(&self) -> &RetryConfig {
        self.retry.config()
    }

    /// 验证单个响应片段
    pub fn validate_chunk(&self, chunk: &str) -> Result<(), ResponseError> {
        if chunk.trim().is_empty() {
            return Err(ResponseError::Empty);
        }

        if chunk.contains("error:") || chunk.contains("Error:") {
            return Err(ResponseError::HasErrorMarker);
        }

        if chunk.len() < 3 {
            return Err(ResponseError::TooShort);
        }

        Ok(())
    }

    /// 将所有片段合并为完整响应
    pub fn finalize_response(&self, chunks: &[String]) -> String {
        chunks.join("")
    }

    /// 对整个响应执行带重试的验证
    pub async fn validate_with_retry<F, Fut>(
        &self,
        mut produce_response: F,
    ) -> Result<String, ResponseError>
    where
        F: FnMut(u32) -> Fut,
        Fut: std::future::Future<Output = Result<String, ResponseError>>,
    {
        self.retry
            .execute_with_retry(|attempt| {
                let fut = produce_response(attempt);
                async move {
                    let response = fut.await?;
                    self.validate_chunk(&response)?;
                    Ok(response)
                }
            })
            .await
    }
}
