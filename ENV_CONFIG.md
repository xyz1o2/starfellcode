# Ghost Text Editor - LLM Configuration Guide

## 快速配置指南

### 1. OpenAI 配置
```bash
LLM_PROVIDER=openai
OPENAI_API_KEY=sk-your-actual-api-key-here
OPENAI_MODEL=gpt-3.5-turbo
```

### 2. Gemini 配置
```bash
LLM_PROVIDER=gemini
GEMINI_API_KEY=your-gemini-api-key-here
GEMINI_MODEL=gemini-1.5-flash
```

### 3. Claude 配置
```bash
LLM_PROVIDER=claude
ANTHROPIC_API_KEY=your-anthropic-api-key-here
CLAUDE_MODEL=claude-3-sonnet
```

### 4. Ollama 本地配置
```bash
LLM_PROVIDER=ollama
OLLAMA_MODEL=mistral
OLLAMA_BASE_URL=http://localhost:11434/api/chat
```

### 5. 本地服务器配置
```bash
LLM_PROVIDER=local
LOCAL_MODEL=liquid/lfm2-1.2b
LOCAL_SERVER_URL=http://localhost:1234/v1/chat/completions
```

## 使用步骤

1. 编辑 `.env` 文件
2. 取消注释你想使用的提供商配置
3. 填入你的 API 密钥
4. 重新运行程序：`cargo run`

## 支持的命令

- `/help` - 显示帮助信息
- `/clear` - 清除聊天历史
- `/status` - 显示当前状态
- `/model` - 显示当前模型信息
- `/provider` - 显示当前提供商
- `/temperature` - 显示当前温度设置
- `/history` - 显示聊天历史

## 注意事项

- 确保 API 密钥有效且有足够的配额
- 本地模型需要先启动对应的服务
- 修改配置后需要重启程序