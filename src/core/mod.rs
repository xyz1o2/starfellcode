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