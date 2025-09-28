# acommit

A minimalist CLI tool that generates intelligent git commit messages using AI (Gemini, Ollama, or OpenAI-compatible APIs).

## Features

- Multiple AI providers: Gemini, Ollama, OpenAI-compatible APIs
- Generates conventional commit messages
- Interactive setup with `--setup`
- Auto-detects `acommit.json` in current directory
- Flexible JSON configuration files
- Smart defaults and fallbacks
- Interactive confirmation before committing
- Verbose mode for debugging

## Installation

### From Source

```bash
git clone https://github.com/skorotkiewicz/acommit
cd acommit
cargo build --release
# Add target/release/acommit to your PATH
```

### Using Cargo

```bash
cargo install --git https://github.com/skorotkiewicz/acommit acommit
```

## Quick Start

```bash
# First time setup - generates acommit.json
acommit --setup

# Daily usage - auto-detects acommit.json
acommit

# Use specific provider from config
acommit --provider ollama
```

## Usage

### Configuration Methods

1. Interactive Setup (Recommended for beginners):
   ```bash
   acommit --setup
   ```

2. Auto-detection: Automatically finds `acommit.json` in current directory

3. Manual Configuration: Use command-line flags

4. Environment Variables: Set `ACOMMIT_CONFIG` for global config

### Command Line Options

- `--setup`: Interactive setup and generate `acommit.json`
- `--config <PATH>`: Use specific configuration file
- `--provider <PROVIDER>`: Override default provider (gemini, ollama, openai)
- `--example-config`: Show example configuration format
- `--verbose`: Show debug information
- `--gemini-key, -gk <KEY>`: Use Gemini API with provided key
- `--ollama-url, -ou <URL>`: Use Ollama at specified URL
- `--openai <URL>`: Use OpenAI-compatible API at specified URL
- `--openai-key, -ok <KEY>`: API key for OpenAI-compatible API (optional)
- `--model, -m <MODEL>`: Model name to use
- `--help, -h`: Show help

### Examples

```bash
# Interactive setup
acommit --setup

# Auto-detect local config
acommit

# Use specific config file
acommit --config my-config.json

# Override provider from config
acommit --provider ollama

# Manual configuration (legacy)
acommit --openai http://localhost:7777/v1 --model bitnet-model
acommit --gemini-key YOUR_API_KEY --model gemini-2.5-flash
acommit --ollama-url http://localhost:11434 --model llama3.2:3b

# Show example config
acommit --example-config
```

## Configuration

### Configuration File Format

The `acommit.json` configuration file supports all providers:

```json
{
  "default_provider": "openai",
  "verbose": false,
  "gemini": {
    "model": "gemini-2.5-flash-lite",
    "api_key": "your-gemini-key"
  },
  "ollama": {
    "model": "llama3.2:3b",
    "url": "http://localhost:11434"
  },
  "openai": {
    "model": "bitnet-model",
    "url": "http://localhost:7777/v1",
    "api_key": "your-openai-key"
  }
}
```

### Configuration Priority

1. `--config <PATH>` (highest priority)
2. `ACOMMIT_CONFIG` environment variable
3. Local `acommit.json` (auto-detected)
4. Default Ollama configuration (fallback)

### Environment Variables

- `ACOMMIT_CONFIG`: Path to default configuration file
- `GEMINI_API_KEY`: Fallback Gemini API key
- `OPENAI_API_KEY`: Fallback OpenAI API key

### Default Models

- Gemini: `gemini-2.5-flash-lite`
- Ollama: `llama3.2:3b`
- OpenAI: `bitnet-model`

## Requirements

- Rust 1.70+
- Git
- For Gemini: API key from Google AI Studio
- For Ollama: Running Ollama instance with compatible model
- For OpenAI-compatible APIs: Compatible endpoint (API key optional)

## How It Works

1. Configuration: Loads config from file, environment, or uses defaults
2. Change Detection: Checks git status for staged/unstaged changes
3. Diff Generation: Creates diff of modified files
4. AI Processing: Sends diff to selected AI provider
5. Message Generation: Creates conventional commit message
6. User Confirmation: Shows generated message and asks for approval
7. Commit Creation: Stages all changes and creates commit

## Supported AI Providers

### Gemini
- API: Google Gemini API
- Authentication: API key required
- Models: `gemini-2.5-flash-lite`, `gemini-pro`, etc.

### Ollama
- API: Local Ollama instance
- Authentication: None required
- Models: Any Ollama model (`llama3.2:3b`, `codellama:7b`, etc.)

### OpenAI-Compatible
- API: OpenAI-compatible endpoints
- Authentication: Optional API key
- Models: Any compatible model (`gpt-4`, `bitnet-model`, etc.)

## Development

This project is an excellent example for learning Rust! It covers:

- Error Handling: `Result<T, E>`, `?` operator
- Pattern Matching: `match` expressions, `Option<T>`
- Ownership & Borrowing: `String` vs `&str`, `clone()`
- Structs & Enums: Complex data structures
- Serde: JSON serialization/deserialization
- Async/Await: HTTP requests with `reqwest`
- CLI Tools: Command-line argument parsing
- File I/O: Configuration file handling
- Interactive CLI: User input with `dialoguer`

### Building from Source

```bash
git clone https://github.com/skorotkiewicz/acommit
cd acommit
cargo build --release
```

### Running Tests

```bash
cargo test
```

## Contributing

Contributions are welcome! This project is perfect for:
- Learning Rust concepts
- Adding new AI providers
- Improving error handling
- Adding new features

## License

MIT
