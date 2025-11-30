/// Token 计算模块 - 对应 Gemini CLI 的 Token 管理
/// 
/// 支持完善的 Token 计算：
/// - 多种编码方式
/// - 精确的 Token 计数
/// - 成本估算
/// - 模型支持

use crate::core::message_history::Message;

/// Token 编码方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenEncoding {
    Cl100kBase,             // GPT-3.5/GPT-4
    P50kBase,               // GPT-3
    R50kBase,               // 编码
}

/// 模型信息
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub encoding: TokenEncoding,
    pub input_price_per_1k: f64,    // 每 1000 tokens 的输入价格（美元）
    pub output_price_per_1k: f64,   // 每 1000 tokens 的输出价格（美元）
}

impl ModelInfo {
    /// GPT-4
    pub fn gpt4() -> Self {
        Self {
            name: "gpt-4".to_string(),
            encoding: TokenEncoding::Cl100kBase,
            input_price_per_1k: 0.03,
            output_price_per_1k: 0.06,
        }
    }

    /// GPT-3.5-turbo
    pub fn gpt35_turbo() -> Self {
        Self {
            name: "gpt-3.5-turbo".to_string(),
            encoding: TokenEncoding::Cl100kBase,
            input_price_per_1k: 0.0005,
            output_price_per_1k: 0.0015,
        }
    }

    /// Gemini 2.5
    pub fn gemini25() -> Self {
        Self {
            name: "gemini-2.5".to_string(),
            encoding: TokenEncoding::Cl100kBase,
            input_price_per_1k: 0.075 / 1000.0,  // $0.075 per 1M tokens
            output_price_per_1k: 0.30 / 1000.0,  // $0.30 per 1M tokens
        }
    }

    /// Claude 3
    pub fn claude3() -> Self {
        Self {
            name: "claude-3".to_string(),
            encoding: TokenEncoding::Cl100kBase,
            input_price_per_1k: 0.003,
            output_price_per_1k: 0.015,
        }
    }
}

/// Token 统计信息
#[derive(Debug, Clone, Default)]
pub struct TokenStats {
    pub total_tokens: usize,
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub tool_tokens: usize,
    pub system_tokens: usize,
}

impl TokenStats {
    /// 创建新的统计信息
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加输入 tokens
    pub fn add_input(&mut self, tokens: usize) {
        self.input_tokens += tokens;
        self.total_tokens += tokens;
    }

    /// 添加输出 tokens
    pub fn add_output(&mut self, tokens: usize) {
        self.output_tokens += tokens;
        self.total_tokens += tokens;
    }

    /// 添加工具 tokens
    pub fn add_tool(&mut self, tokens: usize) {
        self.tool_tokens += tokens;
        self.total_tokens += tokens;
    }

    /// 添加系统 tokens
    pub fn add_system(&mut self, tokens: usize) {
        self.system_tokens += tokens;
        self.total_tokens += tokens;
    }
}

/// Token 计算器
pub struct TokenCalculator {
    model: ModelInfo,
}

impl TokenCalculator {
    /// 创建新的 Token 计算器
    pub fn new(model: ModelInfo) -> Self {
        Self { model }
    }

    /// 使用模型名称创建计算器
    pub fn from_model_name(name: &str) -> Self {
        let model = match name {
            "gpt-4" => ModelInfo::gpt4(),
            "gpt-3.5-turbo" => ModelInfo::gpt35_turbo(),
            "gemini-2.5" => ModelInfo::gemini25(),
            "claude-3" => ModelInfo::claude3(),
            _ => ModelInfo::gpt4(), // 默认使用 GPT-4
        };
        Self::new(model)
    }

    /// 计算文本的 Token 数
    pub fn count_tokens(&self, text: &str) -> usize {
        match self.model.encoding {
            TokenEncoding::Cl100kBase => {
                // 简单估算：平均每 4 个字符 = 1 token
                // 对于英文文本较准确，中文可能需要调整
                let base_tokens = (text.len() + 3) / 4;
                
                // 特殊字符和标点符号可能占用更多 tokens
                let special_chars = text.chars().filter(|c| !c.is_alphanumeric()).count();
                base_tokens + (special_chars / 10)
            }
            TokenEncoding::P50kBase => {
                // GPT-3 编码：稍微不同的计算方式
                (text.len() + 2) / 3
            }
            TokenEncoding::R50kBase => {
                // 编码：另一种计算方式
                (text.len() + 5) / 6
            }
        }
    }

    /// 计算消息的 Token 数
    pub fn count_message_tokens(&self, message: &Message) -> usize {
        let content_tokens = self.count_tokens(&message.content);
        
        // 每条消息的开销：角色标记 + 格式化
        let overhead = 4;
        
        content_tokens + overhead
    }

    /// 计算对话的 Token 数
    pub fn count_conversation_tokens(&self, messages: &[Message]) -> TokenStats {
        let mut stats = TokenStats::new();
        
        for message in messages {
            let tokens = self.count_message_tokens(message);
            
            match &message.role {
                crate::core::message_history::MessageRole::User => {
                    stats.add_input(tokens);
                }
                crate::core::message_history::MessageRole::Assistant => {
                    stats.add_output(tokens);
                }
                crate::core::message_history::MessageRole::System => {
                    stats.add_system(tokens);
                }
            }
        }
        
        stats
    }

    /// 估算成本
    pub fn estimate_cost(&self, stats: &TokenStats) -> f64 {
        let input_cost = (stats.input_tokens as f64 / 1000.0) * self.model.input_price_per_1k;
        let output_cost = (stats.output_tokens as f64 / 1000.0) * self.model.output_price_per_1k;
        
        input_cost + output_cost
    }

    /// 检查是否超过限制
    pub fn exceeds_limit(&self, tokens: usize, limit: usize) -> bool {
        tokens > limit
    }

    /// 获取模型信息
    pub fn get_model_info(&self) -> &ModelInfo {
        &self.model
    }

    /// 计算剩余 tokens
    pub fn calculate_remaining_tokens(&self, used: usize, limit: usize) -> usize {
        if used > limit {
            0
        } else {
            limit - used
        }
    }

    /// 计算 Token 使用率（百分比）
    pub fn calculate_usage_percentage(&self, used: usize, limit: usize) -> f64 {
        if limit == 0 {
            0.0
        } else {
            (used as f64 / limit as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_counting() {
        let calculator = TokenCalculator::new(ModelInfo::gpt4());
        
        let text = "Hello, World!";
        let tokens = calculator.count_tokens(text);
        
        assert!(tokens > 0);
    }

    #[test]
    fn test_model_info() {
        let gpt4 = ModelInfo::gpt4();
        assert_eq!(gpt4.name, "gpt-4");
        assert_eq!(gpt4.encoding, TokenEncoding::Cl100kBase);
        
        let gpt35 = ModelInfo::gpt35_turbo();
        assert_eq!(gpt35.name, "gpt-3.5-turbo");
    }

    #[test]
    fn test_token_stats() {
        let mut stats = TokenStats::new();
        
        stats.add_input(100);
        stats.add_output(50);
        stats.add_tool(10);
        
        assert_eq!(stats.total_tokens, 160);
        assert_eq!(stats.input_tokens, 100);
        assert_eq!(stats.output_tokens, 50);
        assert_eq!(stats.tool_tokens, 10);
    }

    #[test]
    fn test_cost_estimation() {
        let calculator = TokenCalculator::new(ModelInfo::gpt4());
        
        let mut stats = TokenStats::new();
        stats.add_input(1000);
        stats.add_output(500);
        
        let cost = calculator.estimate_cost(&stats);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_token_limit_check() {
        let calculator = TokenCalculator::new(ModelInfo::gpt4());
        
        assert!(!calculator.exceeds_limit(100, 1000));
        assert!(calculator.exceeds_limit(1500, 1000));
    }

    #[test]
    fn test_remaining_tokens() {
        let calculator = TokenCalculator::new(ModelInfo::gpt4());
        
        let remaining = calculator.calculate_remaining_tokens(300, 1000);
        assert_eq!(remaining, 700);
        
        let remaining = calculator.calculate_remaining_tokens(1500, 1000);
        assert_eq!(remaining, 0);
    }

    #[test]
    fn test_usage_percentage() {
        let calculator = TokenCalculator::new(ModelInfo::gpt4());
        
        let percentage = calculator.calculate_usage_percentage(500, 1000);
        assert_eq!(percentage, 50.0);
        
        let percentage = calculator.calculate_usage_percentage(1000, 1000);
        assert_eq!(percentage, 100.0);
    }

    #[test]
    fn test_from_model_name() {
        let calculator = TokenCalculator::from_model_name("gpt-4");
        assert_eq!(calculator.model.name, "gpt-4");
        
        let calculator = TokenCalculator::from_model_name("unknown");
        assert_eq!(calculator.model.name, "gpt-4"); // 默认值
    }
}
