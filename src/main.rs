use std::env;
use std::process::Command;
use std::io::{self, Write};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
enum ModelProvider {
    Gemini { api_key: String, model: String },
    Ollama { base_url: String, model: String },
    OpenAI { base_url: String, api_key: Option<String>, model: String },
}

// Gemini API structures
#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: Option<GeminiResponseContent>,
}

#[derive(Deserialize)]
struct GeminiResponseContent {
    parts: Option<Vec<GeminiResponsePart>>,
}

#[derive(Deserialize)]
struct GeminiResponsePart {
    text: Option<String>,
}

// Ollama API structures
#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: Option<String>,
    // done: Option<bool>,
}

// OpenAI API structures
#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

#[derive(Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Option<Vec<OpenAIChoice>>,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: Option<OpenAIResponseMessage>,
}

#[derive(Deserialize)]
struct OpenAIResponseMessage {
    content: Option<String>,
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("❌ Error: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (config, _verbose) = parse_args()?;
    
    // Debug info
    match &config {
        ModelProvider::Gemini { model, .. } => println!("🧠 Using Gemini model: {}", model),
        ModelProvider::Ollama { base_url, model } => println!("🦙 Using Ollama model: {} at {}", model, base_url),
        ModelProvider::OpenAI { base_url, model, .. } => println!("🤖 Using OpenAI model: {} at {}", model, base_url),
    }

    println!("🔍 Checking git status...");
    
    // Check if we're in a git repository
    let status = Command::new("git")
        .args(&["status", "--porcelain"])
        .output()?;

    if !status.status.success() {
        return Err("Not a git repository or git not found".into());
    }

    let changes = String::from_utf8_lossy(&status.stdout);
    if changes.trim().is_empty() {
        println!("✅ No changes to commit");
        return Ok(());
    }

    println!("📝 Found changes:");
    for line in changes.lines().take(10) {
        println!("  {}", line);
    }

    // Get diff for AI
    let diff_output = Command::new("git")
        .args(&["diff", "--cached", "--name-status"])
        .output()?;

    let mut diff_info = String::from_utf8_lossy(&diff_output.stdout).to_string();
    
    // If there are no staged changes, show all changes
    if diff_info.trim().is_empty() {
        let all_diff = Command::new("git")
            .args(&["diff", "--name-status"])
            .output()?;
        diff_info = String::from_utf8_lossy(&all_diff.stdout).to_string();
    }

    println!("🤖 Generating commit message with AI...");
    
    // Create prompt for AI
    let prompt = format!(
        "Generate a concise, clear git commit message in English based on these file changes:\n\n{}\n\nRules:\n- Use conventional commits format (feat:, fix:, docs:, etc.)\n- Be specific but concise\n- Maximum 50 characters for the title\n- Only return the commit message, nothing else",
        diff_info.trim()
    );

    // Call the appropriate API
    let commit_message = match config {
        ModelProvider::Gemini { api_key, model } => {
            call_gemini_api(&api_key, &model, &prompt).await?
        },
        ModelProvider::Ollama { base_url, model } => {
            call_ollama_api(&base_url, &model, &prompt).await?
        },
        ModelProvider::OpenAI { base_url, api_key, model } => {
            call_openai_api(&base_url, api_key.as_ref(), &model, &prompt).await?
        },
    };
    
    println!("📋 Generated commit message: {}", commit_message);
    
    // Ask user for confirmation
    print!("🤔 Use this commit message? (y/N): ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    if !input.trim().to_lowercase().starts_with('y') {
        println!("❌ Commit cancelled");
        return Ok(());
    }

    // Execute git add -A
    println!("➕ Adding all changes...");
    let add_status = Command::new("git")
        .args(&["add", "-A"])
        .status()?;

    if !add_status.success() {
        return Err("Failed to add changes".into());
    }

    // Execute commit
    println!("💾 Creating commit...");
    let commit_status = Command::new("git")
        .args(&["commit", "-m", &commit_message])
        .status()?;

    if commit_status.success() {
        println!("✅ Successfully committed with message: {}", commit_message);
    } else {
        return Err("Failed to create commit".into());
    }

    Ok(())
}

fn parse_args() -> Result<(ModelProvider, bool), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    // Check for help flags
    for arg in &args[1..] {
        if arg == "--help" || arg == "-h" {
            print_usage();
            std::process::exit(0);
        }
    }
    
    let mut gemini_api_key = None;
    let mut ollama_url = None;
    let mut openai_url = None;
    let mut openai_api_key = None;
    let mut model_name = None;
    let mut verbose = false;
    
    // Parse arguments
    for arg in args.iter().skip(1) {
        if let Some((key, value)) = arg.split_once('=') {
            match key {
                "--gemini-key" | "-gk" => gemini_api_key = Some(value.to_string()),
                "--ollama-url" | "-ou" => ollama_url = Some(value.to_string()),
                "--openai" => openai_url = Some(value.to_string()),
                "--openai-key" | "-ok" => openai_api_key = Some(value.to_string()),
                "--model" | "-m" => model_name = Some(value.to_string()),
                "--verbose" => verbose = true,
                _ => return Err(format!("Unknown argument: {}", key).into()),
            }
        } else {
            // Handle space-separated arguments
            match arg.as_str() {
                "--gemini-key" | "-gk" => {
                    if let Some(next_arg) = args.iter().skip_while(|a| *a != arg).nth(1) {
                        gemini_api_key = Some(next_arg.clone());
                    }
                },
                "--ollama-url" | "-ou" => {
                    if let Some(next_arg) = args.iter().skip_while(|a| *a != arg).nth(1) {
                        ollama_url = Some(next_arg.clone());
                    }
                },
                "--openai" => {
                    if let Some(next_arg) = args.iter().skip_while(|a| *a != arg).nth(1) {
                        openai_url = Some(next_arg.clone());
                    }
                },
                "--openai-key" | "-ok" => {
                    if let Some(next_arg) = args.iter().skip_while(|a| *a != arg).nth(1) {
                        openai_api_key = Some(next_arg.clone());
                    }
                },
                "--model" | "-m" => {
                    if let Some(next_arg) = args.iter().skip_while(|a| *a != arg).nth(1) {
                        model_name = Some(next_arg.clone());
                    }
                },
                "--verbose" => verbose = true,
                _ => {} // Skip unknown single arguments
            }
        }
    }
    
    // Debug output (only if verbose)
    if verbose {
        eprintln!("Debug - gemini_api_key: {:?}", gemini_api_key);
        eprintln!("Debug - ollama_url: {:?}", ollama_url);
        eprintln!("Debug - openai_url: {:?}", openai_url);
        eprintln!("Debug - openai_api_key: {:?}", openai_api_key);
        eprintln!("Debug - model_name: {:?}", model_name);
    }
    
    // Determine provider and configuration
    if let Some(url) = openai_url {
        // OpenAI explicitly specified
        let api_key = openai_api_key
            .or_else(|| env::var("OPENAI_API_KEY").ok());
        Ok((ModelProvider::OpenAI { 
            base_url: url, 
            api_key,
            model: model_name.unwrap_or_else(|| "gpt-3.5-turbo".to_string())
        }, verbose))
    } else if let Some(url) = ollama_url {
        // Ollama explicitly specified
        Ok((ModelProvider::Ollama { 
            base_url: url, 
            model: model_name.unwrap_or_else(|| "llama3.2:3b".to_string())
        }, verbose))
    } else if let Some(key) = gemini_api_key {
        // Gemini key explicitly specified
        Ok((ModelProvider::Gemini { 
            api_key: key, 
            model: model_name.unwrap_or_else(|| "gemini-2.5-flash-lite".to_string())
        }, verbose))
    } else {
        // No explicit provider, check environment and defaults
        if let Ok(api_key) = env::var("GEMINI_API_KEY") {
            Ok((ModelProvider::Gemini { 
                api_key, 
                model: model_name.unwrap_or_else(|| "gemini-2.5-flash-lite".to_string())
            }, verbose))
        } else {
            // Default to Ollama
            Ok((ModelProvider::Ollama { 
                base_url: "http://localhost:11434".to_string(),
                model: model_name.unwrap_or_else(|| "llama3.2:3b".to_string())
            }, verbose))
        }
    }
}

async fn call_gemini_api(api_key: &str, model: &str, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    let request_body = GeminiRequest {
        contents: vec![GeminiContent {
            parts: vec![GeminiPart {
                text: prompt.to_string(),
            }],
        }],
    };

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("Gemini API request failed: {}", response.status()).into());
    }

    let data: GeminiResponse = response.json().await?;
    
    let commit_message = data
        .candidates
        .and_then(|candidates| candidates.into_iter().next())
        .and_then(|candidate| candidate.content)
        .and_then(|content| content.parts)
        .and_then(|parts| parts.into_iter().next())
        .and_then(|part| part.text)
        .unwrap_or_else(|| "chore: update files".to_string())
        .trim()
        .to_string();

    // Remove unnecessary whitespace
    let clean_message = commit_message
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ");

    Ok(clean_message)
}

async fn call_ollama_api(base_url: &str, model: &str, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    let request_body = OllamaRequest {
        model: model.to_string(),
        prompt: prompt.to_string(),
        stream: false,
    };

    let url = format!("{}/api/generate", base_url);

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("Ollama API request failed: {}", response.status()).into());
    }

    let data: OllamaResponse = response.json().await?;
    
    let commit_message = data
        .response
        .unwrap_or_else(|| "chore: update files".to_string())
        .trim()
        .to_string();

    // Remove unnecessary whitespace and take the first line (sometimes Ollama returns more)
    let clean_message = commit_message
        .lines()
        .next()
        .unwrap_or("chore: update files")
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ");

    Ok(clean_message)
}

async fn call_openai_api(base_url: &str, api_key: Option<&String>, model: &str, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    let request_body = OpenAIRequest {
        model: model.to_string(),
        messages: vec![OpenAIMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }],
        max_tokens: Some(100),
        temperature: Some(0.7),
    };

    let url = format!("{}/chat/completions", base_url);

    let mut request = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body);

    // Add Authorization header only if API key is provided
    if let Some(key) = api_key {
        request = request.header("Authorization", format!("Bearer {}", key));
    }

    let response = request.send().await?;

    if !response.status().is_success() {
        return Err(format!("OpenAI API request failed: {}", response.status()).into());
    }

    let data: OpenAIResponse = response.json().await?;
    
    let commit_message = data
        .choices
        .and_then(|choices| choices.into_iter().next())
        .and_then(|choice| choice.message)
        .and_then(|message| message.content)
        .unwrap_or_else(|| "chore: update files".to_string())
        .trim()
        .to_string();

    // Remove unnecessary whitespace and take the first line
    let clean_message = commit_message
        .lines()
        .next()
        .unwrap_or("chore: update files")
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ");

    Ok(clean_message)
}

fn print_usage() {
    println!("Usage: acommit [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("  --gemini-key, -gk <KEY>     Use Gemini API with provided key");
    println!("  --ollama-url, -ou <URL>     Use Ollama at specified URL");
    println!("  --openai <URL>              Use OpenAI-compatible API at specified URL");
    println!("  --openai-key, -ok <KEY>     API key for OpenAI-compatible API (optional)");
    println!("  --model, -m <MODEL>         Model name to use");
    println!("  --verbose                   Show debug information");
    println!();
    println!("Examples:");
    println!("  acommit # Use GEMINI_API_KEY env var or default Ollama");
    println!("  acommit --ollama-url http://localhost:11434       # Use local Ollama");
    println!("  acommit --openai http://localhost:8080/v1 --model bitnet-model # Use OpenAI-compatible API");
    println!("  acommit --openai http://api.openai.com/v1 --openai-key sk-xxx --model gpt-4 # Use OpenAI with API key");
    println!("  acommit --model llama3.2:3b                       # Specify model");
    println!("  acommit --gemini-key xyz --model gemini-2.5-flash # Use Gemini with specific key");
    println!("  acommit -ou http://server:11434 -m codellama:7b   # Remote Ollama with CodeLlama");
    println!("  acommit --verbose --openai http://localhost:8080/v1 # Show debug info");
    println!();
    println!("Environment Variables:");
    println!("  GEMINI_API_KEY              Used as fallback if no provider specified");
    println!("  OPENAI_API_KEY              Used for OpenAI-compatible APIs when --openai-key not provided");
}