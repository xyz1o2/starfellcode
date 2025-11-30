pub mod buffer;
pub mod cursor;
pub mod history;
pub mod message;
pub mod context_optimizer;
pub mod integration;
pub mod conversation_engine;
pub mod gemini_architecture;
pub mod retry_handler;
pub mod response_validation;
pub mod tool_executor;
pub mod routing_strategy;
pub mod streaming;
pub mod hooks;
pub mod routing_strategies;
pub mod message_history;
pub mod error_recovery;
pub mod streaming_optimizer;
pub mod token_calculator;
pub mod logger;
pub mod performance_optimizer;
pub mod test_suite;

pub use conversation_engine::{
    ConversationEngine, IntentRecognizer, ContextManager, ResponseProcessor,
    UserIntent, ConversationContext, ProcessedResponse, CodeModification,
    ModificationOperation, FileContent,
};

pub use gemini_architecture::{
    GeminiArchitecture, CompositeRouter,
    Turn, ConversationHistory, StreamEventType,
};

pub use retry_handler::{RetryConfig, RetryHandler, RetryableError};
pub use response_validation::{ResponseValidator, ResponseError};
pub use routing_strategy::{RoutingDecision, RoutingStrategy};
pub use streaming::{StreamEvent, StreamEventType as StreamType, StreamHandler, StreamBuffer};
pub use hooks::{HookManager, BeforeModelHook, AfterModelHook, BeforeToolSelectionHook, 
    AfterToolExecutionHook, OnRetryHook, LoggingHook};
pub use routing_strategies::{FallbackStrategy, ModelSelectionStrategy, CostOptimizationStrategy, PerformanceStrategy};
pub use message_history::{MessageHistory, Message, MessageRole, Turn as MessageTurn};
pub use error_recovery::{ErrorRecovery, RecoverableError, RecoveryStrategy, RecoveryConfig, RecoveryResult};
pub use streaming_optimizer::{StreamingOptimizer, StreamingOptimizerConfig, OptimizedStreamEvent, PerformanceMetrics};
pub use token_calculator::{TokenCalculator, TokenEncoding, ModelInfo, TokenStats};