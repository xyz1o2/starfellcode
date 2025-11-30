# Starfellcode - AI-Powered Terminal Code Editor

🤖 A modern terminal-based AI code editor with pixel art aesthetics, built with Rust and Ratatui.

## Features

- **AI-Powered Pair Programming**: Integrated LLM support for code suggestions and modifications
- **Pixel Art UI**: Retro terminal interface with 8x8 pixel avatars
- **File Operations**: Create, edit, and manage files directly from the terminal
- **Streaming Responses**: Real-time AI response streaming
- **Code Modification Confirmation**: Safe AI-suggested code changes with user approval
- **File Search & @Mentions**: Quick file referencing with @filename syntax

## Quick Start

### Prerequisites

- Rust 1.70+
- LLM API access (OpenAI, Anthropic, etc.)

### Installation

```bash
git clone https://github.com/yourusername/starfellcode.git
cd starfellcode
cargo build --release
```

### Configuration

Create a `.env` file in the project root:

```env
# OpenAI
OPENAI_API_KEY=your_openai_api_key
OPENAI_MODEL=gpt-4

# Or Anthropic
ANTHROPIC_API_KEY=your_anthropic_key
ANTHROPIC_MODEL=claude-3-sonnet-20240229

# Or other LLM providers
```

### Usage

```bash
cargo run
```

## Interface Guide

```
┌─────────────────────────────────────────────────────┐
│ 🤖 Starfellcode Pair Programming v0.1.0            │
├─────────────────────────────────────────────────────┤
│ ████  System: Welcome to Starfellcode!             │
│ ████  AI: Hello! How can I help you code today?    │
│ ████  You: @src/main.rs Show me the main function  │
├─────────────────────────────────────────────────────┤
│ > Type your message here...                         │
├─────────────────────────────────────────────────────┤
│ Ready | ESC: Quit | Tab: Commands                   │
└─────────────────────────────────────────────────────┘
```

### Commands

- `@filename` - Reference and include file content in your message
- `/help` - Show available commands
- `/clear` - Clear chat history
- `/edit filename` - Edit a file
- `/create filename` - Create a new file

## Architecture

- **UI Layer**: Pixel art layout with Ratatui
- **AI Layer**: Streaming LLM integration with multiple providers
- **Core Layer**: Message handling, file operations, and state management
- **Commands Layer**: File manipulation and system commands

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Code Style

```bash
cargo fmt
cargo clippy
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

MIT License - see LICENSE file for details

## Acknowledgments

- Inspired by GitHub Copilot and modern AI coding assistants
- Built with [Ratatui](https://github.com/ratatui-org/ratatui) for terminal UI
- Pixel art design inspired by retro gaming aesthetics</content>
<parameter name="filePath">/mnt/g/ai-workspace/starfellcode/README.md