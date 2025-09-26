# acommit

A minimalist CLI tool that generates intelligent git commit messages using AI (Gemini or Ollama).

## Features

- Automatically detects git changes
- Generates conventional commit messages
- Supports Gemini API and Ollama
- Interactive confirmation before committing
- Works with staged or unstaged changes

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

## Usage

### Basic Usage

```bash
# Use default provider (GEMINI_API_KEY env var or Ollama at localhost:11434)
acommit

# Use Ollama locally
acommit --ollama-url http://localhost:11434

# Use Gemini with API key
acommit --gemini-key YOUR_API_KEY

# Specify model
acommit --model llama3.2:3b
```

### Options

- `--gemini-key, -gk <KEY>`: Use Gemini API with provided key
- `--ollama-url, -ou <URL>`: Use Ollama at specified URL
- `--model, -m <MODEL>`: Model name to use
- `--help, -h`: Show help

### Examples

```bash
# Default (checks GEMINI_API_KEY env var, falls back to Ollama)
acommit

# Local Ollama with specific model
acommit -ou http://localhost:11434 -m llama3.2:3b

# Gemini with custom model
acommit -gk YOUR_API_KEY -m gemini-2.5-flash

# Remote Ollama server
acommit -ou http://server:11434 -m codellama:7b
```

## Configuration

### Environment Variables

- `GEMINI_API_KEY`: Your Google Gemini API key (optional, will use Ollama if not set)

### Default Models

- Gemini: `gemini-2.5-flash-lite`
- Ollama: `llama3.2:3b`

## Requirements

- Rust 1.70+
- Git
- For Gemini: API key from Google AI Studio
- For Ollama: Running Ollama instance with compatible model

## How It Works

1. Checks git status for changes
2. Gets diff of modified files
3. Sends diff to AI for commit message generation
4. Shows generated message and asks for confirmation
5. Stages all changes and creates commit

## License

MIT
