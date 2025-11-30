/// Gemini CLI 核心架构的 Rust 实现
/// 
/// 这个模块实现了 Gemini CLI 的核心设计模式：
/// 1. 流式处理 + 重试机制
/// 2. 工具调用的递归处理
/// 3. 路由策略模式
/// 4. 内容验证
/// 5. 对话轮次管理

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use crate::ai::client::{ChatMessage, LLMClient};
use crate::ai::prompt_builder::{Message as PromptMessage, PromptBuilder};
use futures_util::future::BoxFuture;

// ============================================================================
// 1. 流式处理 + 重试机制
// ============================================================================

/// 流式事件类型（参考 Gemini CLI 的 StreamEventType）
#[derive(Debug, Clone)]
pub enum StreamEventType {
    /// 普通内容块
    Chunk(String),
    /// 重试信号
    Retry,
    /// 完成
    Complete,
}

/// 重试配置（参考 Gemini CLI 的 INVALID_CONTENT_RETRY_OPTIONS）
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// 最大重试次数（1 初始 + N 重试）
    pub max_attempts: u32,
    /// 初始延迟（毫秒）
    pub initial_delay_ms: u64,
    /// 退避倍数
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 2,
            initial_delay_ms: 500,
            backoff_multiplier: 2.0,
        }
    }
}

/// 响应验证器（参考 Gemini CLI 的 isValidResponse）
pub struct ResponseValidator {
    config: RetryConfig,
}

impl ResponseValidator {
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// 验证响应是否有效
    pub fn is_valid_response(&self, response: &str) -> bool {
        // 1. 检查响应不为空
        if response.is_empty() {
            return false;
        }

        // 2. 检查是否包含候选项（LLM 返回的内容）
        if response.len() < 10 {
            return false;
        }

        // 3. 检查是否包含有效内容
        self.is_valid_content(response)
    }

    /// 检查内容有效性
    fn is_valid_content(&self, content: &str) -> bool {
        // 检查是否有实际内容（不只是错误消息）
        !content.contains("error") || content.contains("error handling")
    }

    pub fn config(&self) -> &RetryConfig {
        &self.config
    }

    /// 带重试的响应处理
    pub async fn validate_with_retry<F, T>(
        &self,
        mut operation: F,
    ) -> Result<T, String>
    where
        F: FnMut() -> BoxFuture<'static, Result<T, String>>,
    {
        let mut delay = self.config.initial_delay_ms;

        for attempt in 0..self.config.max_attempts {
            match operation().await {
                Ok(result) => {
                    if attempt > 0 {
                        println!("✓ Succeeded on attempt {}", attempt + 1);
                    }
                    return Ok(result);
                }
                Err(e) if attempt < self.config.max_attempts - 1 => {
                    println!(
                        "⚠ Attempt {} failed: {}. Retrying in {}ms...",
                        attempt + 1,
                        e,
                        delay
                    );
                    sleep(Duration::from_millis(delay)).await;
                    delay = (delay as f64 * self.config.backoff_multiplier) as u64;
                }
                Err(e) => {
                    println!("✗ All {} attempts failed: {}", self.config.max_attempts, e);
                    return Err(e);
                }
            }
        }

        unreachable!()
    }
}

// ============================================================================
// 2. 工具调用的递归处理
// ============================================================================

/// 工具调用结果
#[derive(Debug, Clone)]
pub struct ToolCallResult {
    pub tool_name: String,
    pub result: String,
    pub success: bool,
}

/// 工具调度器（参考 Gemini CLI 的 CoreToolScheduler）
pub struct ToolScheduler {
    max_recursion_depth: u32,
}

impl ToolScheduler {
    pub fn new() -> Self {
        Self {
            max_recursion_depth: 5,
        }
    }

    /// 执行工具调用并递归处理结果
    pub async fn execute_and_recurse(
        &self,
        tool_calls: Vec<String>,
        depth: u32,
    ) -> Result<Vec<ToolCallResult>, String> {
        // 1. 检查递归深度
        if depth > self.max_recursion_depth {
            return Err("Max recursion depth exceeded".to_string());
        }

        let mut results = Vec::new();

        // 2. 执行每个工具调用
        for tool_call in tool_calls {
            let result = self.execute_tool(&tool_call).await?;
            results.push(result);
        }

        // 3. 如果有工具调用失败，可以递归重试
        // 这里简化处理，实际应该检查是否需要递归

        Ok(results)
    }

    /// 执行单个工具
    async fn execute_tool(&self, tool_call: &str) -> Result<ToolCallResult, String> {
        // 简化实现，实际应该根据工具名称调用相应的工具
        Ok(ToolCallResult {
            tool_name: tool_call.to_string(),
            result: format!("Executed: {}", tool_call),
            success: true,
        })
    }
}

// ============================================================================
// 3. 路由策略模式
// ============================================================================

/// 路由决策（参考 Gemini CLI 的 RoutingDecision）
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    /// 选择的模型
    pub model: String,
    /// 决策元数据
    pub metadata: RoutingMetadata,
}

#[derive(Debug, Clone)]
pub struct RoutingMetadata {
    pub source: String,
    pub latency_ms: u64,
    pub reasoning: String,
}

/// 路由策略接口（参考 Gemini CLI 的 RoutingStrategy）
#[async_trait::async_trait]
pub trait RoutingStrategy: Send + Sync {
    fn name(&self) -> &str;

    async fn route(
        &self,
        input: &str,
        context: &str,
    ) -> Result<RoutingDecision, String>;
}

/// 基于输入长度的路由策略
pub struct LengthBasedStrategy;

#[async_trait::async_trait]
impl RoutingStrategy for LengthBasedStrategy {
    fn name(&self) -> &str {
        "length_based"
    }

    async fn route(
        &self,
        input: &str,
        _context: &str,
    ) -> Result<RoutingDecision, String> {
        let model = if input.len() > 1000 {
            "gemini-2.5-pro"
        } else {
            "gemini-2.0-flash"
        };

        Ok(RoutingDecision {
            model: model.to_string(),
            metadata: RoutingMetadata {
                source: "length_based".to_string(),
                latency_ms: 0,
                reasoning: format!("Selected {} based on input length ({})", model, input.len()),
            },
        })
    }
}

/// 基于意图的路由策略
pub struct IntentBasedStrategy;

#[async_trait::async_trait]
impl RoutingStrategy for IntentBasedStrategy {
    fn name(&self) -> &str {
        "intent_based"
    }

    async fn route(
        &self,
        _input: &str,
        context: &str,
    ) -> Result<RoutingDecision, String> {
        let model = if context.contains("review") || context.contains("debug") {
            "gemini-2.5-pro"
        } else {
            "gemini-2.0-flash"
        };

        Ok(RoutingDecision {
            model: model.to_string(),
            metadata: RoutingMetadata {
                source: "intent_based".to_string(),
                latency_ms: 0,
                reasoning: format!("Selected {} based on intent", model),
            },
        })
    }
}

/// 组合路由器（参考 Gemini CLI 的 CompositeRouter）
pub struct CompositeRouter {
    strategies: Vec<Box<dyn RoutingStrategy>>,
}

impl CompositeRouter {
    pub fn new() -> Self {
        Self {
            strategies: vec![
                Box::new(IntentBasedStrategy),
                Box::new(LengthBasedStrategy),
            ],
        }
    }

    pub async fn route(
        &self,
        input: &str,
        context: &str,
    ) -> Result<RoutingDecision, String> {
        for strategy in &self.strategies {
            match strategy.route(input, context).await {
                Ok(decision) => {
                    println!("✓ {} -> {}", strategy.name(), decision.model);
                    return Ok(decision);
                }
                Err(_) => continue,
            }
        }

        // 默认策略
        Ok(RoutingDecision {
            model: "gemini-2.5-pro".to_string(),
            metadata: RoutingMetadata {
                source: "default".to_string(),
                latency_ms: 0,
                reasoning: "Default model".to_string(),
            },
        })
    }
}

// ============================================================================
// 4. 对话轮次管理
// ============================================================================

/// 对话轮次（参考 Gemini CLI 的 Turn）
#[derive(Debug, Clone)]
pub struct Turn {
    pub turn_number: u32,
    pub user_input: String,
    pub ai_response: String,
    pub tool_calls: Vec<String>,
    pub tool_results: Vec<ToolCallResult>,
}

impl Turn {
    pub fn new(turn_number: u32, user_input: String) -> Self {
        Self {
            turn_number,
            user_input,
            ai_response: String::new(),
            tool_calls: Vec::new(),
            tool_results: Vec::new(),
        }
    }

    pub fn with_response(mut self, response: String) -> Self {
        self.ai_response = response;
        self
    }

    pub fn with_tool_calls(mut self, calls: Vec<String>) -> Self {
        self.tool_calls = calls;
        self
    }

    pub fn with_tool_results(mut self, results: Vec<ToolCallResult>) -> Self {
        self.tool_results = results;
        self
    }
}

/// 对话历史管理
pub struct ConversationHistory {
    turns: Vec<Turn>,
}

impl ConversationHistory {
    pub fn new() -> Self {
        Self { turns: Vec::new() }
    }

    pub fn add_turn(&mut self, turn: Turn) {
        self.turns.push(turn);
    }

    pub fn get_turns(&self) -> &[Turn] {
        &self.turns
    }

    pub fn get_last_turn(&self) -> Option<&Turn> {
        self.turns.last()
    }

    pub fn get_context(&self) -> String {
        self.turns
            .iter()
            .map(|turn| {
                format!(
                    "Turn {}:\nUser: {}\nAI: {}",
                    turn.turn_number, turn.user_input, turn.ai_response
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

// ============================================================================
// 5. 主对话引擎
// ============================================================================

/// 主对话引擎（参考 Gemini CLI 的 GeminiChat）
pub struct GeminiArchitecture {
    pub validator: ResponseValidator,
    pub scheduler: ToolScheduler,
    pub router: CompositeRouter,
    pub history: ConversationHistory,
    llm_client: Option<Arc<LLMClient>>,
    prompt_builder: PromptBuilder,
    turn_counter: u32,
}

impl GeminiArchitecture {
    pub fn new() -> Self {
        Self {
            validator: ResponseValidator::new(RetryConfig::default()),
            scheduler: ToolScheduler::new(),
            router: CompositeRouter::new(),
            history: ConversationHistory::new(),
            llm_client: None,
            prompt_builder: PromptBuilder::new(),
            turn_counter: 0,
        }
    }

    pub fn set_llm_client(&mut self, client: Arc<LLMClient>) {
        self.llm_client = Some(client);
    }

    pub fn set_prompt_builder(&mut self, builder: PromptBuilder) {
        self.prompt_builder = builder;
    }

    fn build_chat_messages(&self, user_input: &str) -> Vec<ChatMessage> {
        let prompt_messages: Vec<PromptMessage> = self.prompt_builder.build_messages(user_input);
        prompt_messages
            .into_iter()
            .map(|m| ChatMessage {
                role: m.role,
                content: m.content,
            })
            .collect()
    }

    /// 完整的对话流程
    pub async fn chat(&mut self, user_input: String) -> Result<String, String> {
        let llm_client = self
            .llm_client
            .clone()
            .ok_or_else(|| "LLM client is not configured".to_string())?;

        self.turn_counter += 1;
        let mut turn = Turn::new(self.turn_counter, user_input.clone());

        // 1. 路由决策
        let routing_decision = self.router.route(&user_input, &self.history.get_context()).await?;
        println!("📍 Routing: {}", routing_decision.model);

        let messages = self.build_chat_messages(&user_input);

        // 2. 调用 LLM
        let response = self
            .call_llm_with_retry(llm_client, messages, routing_decision.model.clone())
            .await?;
        turn = turn.with_response(response.clone());

        // 3. 验证响应
        if !self.validator.is_valid_response(&response) {
            return Err("Invalid response from LLM".to_string());
        }

        // 4. 检测工具调用
        let tool_calls = self.extract_tool_calls(&response);
        if !tool_calls.is_empty() {
            println!("🔧 Tool calls detected: {:?}", tool_calls);
            turn = turn.with_tool_calls(tool_calls.clone());

            // 5. 执行工具
            let results = self.scheduler.execute_and_recurse(tool_calls, 0).await?;
            turn = turn.with_tool_results(results);
        }

        // 6. 保存到历史
        self.history.add_turn(turn);

        Ok(response)
    }

    async fn call_llm_with_retry(
        &self,
        llm_client: Arc<LLMClient>,
        messages: Vec<ChatMessage>,
        model: String,
    ) -> Result<String, String> {
        let retry_config = self.validator.config().clone();
        let max_attempts = retry_config.max_attempts.max(1);
        let mut attempt = 0;
        let mut delay = retry_config.initial_delay_ms;

        loop {
            use std::sync::Mutex;
            let buffer = std::sync::Arc::new(Mutex::new(String::new()));
            let buffer_clone = buffer.clone();
            let result = llm_client
                .generate_completion_stream(messages.clone(), Some(model.clone()), move |chunk| {
                    let mut buf = buffer_clone.lock().unwrap();
                    buf.push_str(&chunk);
                    true
                })
                .await;

            let buffer_content = buffer.lock().unwrap().clone();
            match result {
                Ok(_) => {
                    if self.validator.is_valid_response(&buffer_content) {
                        return Ok(buffer_content);
                    }
                    attempt += 1;
                    if attempt >= max_attempts {
                        return Err("Invalid response from LLM".to_string());
                    }
                }
                Err(err) => {
                    attempt += 1;
                    if attempt >= max_attempts {
                        return Err(err.to_string());
                    }
                }
            }

            sleep(Duration::from_millis(delay)).await;
            delay = (delay as f64 * retry_config.backoff_multiplier) as u64;
        }
    }

    /// 提取工具调用
    fn extract_tool_calls(&self, response: &str) -> Vec<String> {
        // 简化实现，实际应该解析 LLM 的工具调用格式
        if response.contains("tool") {
            vec!["tool_call_1".to_string()]
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_validator() {
        let validator = ResponseValidator::new(RetryConfig::default());
        assert!(validator.is_valid_response("This is a valid response"));
        assert!(!validator.is_valid_response(""));
        assert!(!validator.is_valid_response("x"));
    }

    #[test]
    fn test_turn_creation() {
        let turn = Turn::new(1, "Hello".to_string())
            .with_response("Hi there".to_string());
        assert_eq!(turn.turn_number, 1);
        assert_eq!(turn.user_input, "Hello");
        assert_eq!(turn.ai_response, "Hi there");
    }

    #[tokio::test]
    async fn test_routing() {
        let router = CompositeRouter::new();
        let decision = router.route("short", "").await.unwrap();
        assert!(!decision.model.is_empty());
    }
}
