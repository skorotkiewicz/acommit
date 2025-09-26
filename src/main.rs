use std::env;
use std::process::Command;
use std::io::{self, Write};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
enum ModelProvider {
    Gemini { api_key: String, model: String },
    Ollama { base_url: String, model: String },
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
    done: Option<bool>,
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("❌ Error: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = parse_args()?;
    
    // Debug info
    match &config {
        ModelProvider::Gemini { model, .. } => println!("🧠 Using Gemini model: {}", model),
        ModelProvider::Ollama { base_url, model } => println!("🦙 Using Ollama model: {} at {}", model, base_url),
    }

    println!("🔍 Checking git status...");
    
    // Sprawdź czy jesteśmy w repozytorium git
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

    // Pobierz diff dla AI
    let diff_output = Command::new("git")
        .args(&["diff", "--cached", "--name-status"])
        .output()?;

    let mut diff_info = String::from_utf8_lossy(&diff_output.stdout).to_string();
    
    // Jeśli nie ma staged changes, pokaż wszystkie zmiany
    if diff_info.trim().is_empty() {
        let all_diff = Command::new("git")
            .args(&["diff", "--name-status"])
            .output()?;
        diff_info = String::from_utf8_lossy(&all_diff.stdout).to_string();
    }

    println!("🤖 Generating commit message with AI...");
    
    // Stwórz prompt dla AI
    let prompt = format!(
        "Generate a concise, clear git commit message in English based on these file changes:\n\n{}\n\nRules:\n- Use conventional commits format (feat:, fix:, docs:, etc.)\n- Be specific but concise\n- Maximum 50 characters for the title\n- Only return the commit message, nothing else",
        diff_info.trim()
    );

    // Wywołaj odpowiednie API
    let commit_message = match config {
        ModelProvider::Gemini { api_key, model } => {
            call_gemini_api(&api_key, &model, &prompt).await?
        },
        ModelProvider::Ollama { base_url, model } => {
            call_ollama_api(&base_url, &model, &prompt).await?
        },
    };
    
    println!("📋 Generated commit message: {}", commit_message);
    
    // Zapytaj użytkownika o potwierdzenie
    print!("🤔 Use this commit message? (y/N): ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    if !input.trim().to_lowercase().starts_with('y') {
        println!("❌ Commit cancelled");
        return Ok(());
    }

    // Wykonaj git add -A
    println!("➕ Adding all changes...");
    let add_status = Command::new("git")
        .args(&["add", "-A"])
        .status()?;

    if !add_status.success() {
        return Err("Failed to add changes".into());
    }

    // Wykonaj commit
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

fn parse_args() -> Result<ModelProvider, Box<dyn std::error::Error>> {
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
    let mut model_name = None;
    
    // Parse arguments
    for arg in args.iter().skip(1) {
        if let Some((key, value)) = arg.split_once('=') {
            match key {
                "--gemini-key" | "-gk" => gemini_api_key = Some(value.to_string()),
                "--ollama-url" | "-ou" => ollama_url = Some(value.to_string()),
                "--model" | "-m" => model_name = Some(value.to_string()),
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
                "--model" | "-m" => {
                    if let Some(next_arg) = args.iter().skip_while(|a| *a != arg).nth(1) {
                        model_name = Some(next_arg.clone());
                    }
                },
                _ => {} // Skip unknown single arguments
            }
        }
    }
    
    // Debug output
    eprintln!("Debug - gemini_api_key: {:?}", gemini_api_key);
    eprintln!("Debug - ollama_url: {:?}", ollama_url);
    eprintln!("Debug - model_name: {:?}", model_name);
    
    // Determine provider and configuration
    if let Some(url) = ollama_url {
        // Ollama explicitly specified
        Ok(ModelProvider::Ollama { 
            base_url: url, 
            model: model_name.unwrap_or_else(|| "llama3.2:3b".to_string())
        })
    } else if let Some(key) = gemini_api_key {
        // Gemini key explicitly specified
        Ok(ModelProvider::Gemini { 
            api_key: key, 
            model: model_name.unwrap_or_else(|| "gemini-2.5-flash-lite".to_string())
        })
    } else {
        // No explicit provider, check environment and defaults
        if let Ok(api_key) = env::var("GEMINI_API_KEY") {
            Ok(ModelProvider::Gemini { 
                api_key, 
                model: model_name.unwrap_or_else(|| "gemini-2.5-flash-lite".to_string())
            })
        } else {
            // Default to Ollama
            Ok(ModelProvider::Ollama { 
                base_url: "http://localhost:11434".to_string(),
                model: model_name.unwrap_or_else(|| "llama3.2:3b".to_string())
            })
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

    // Usuń zbędne białe znaki
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

    // Usuń zbędne białe znaki i weź pierwszą linię (czasami Ollama zwraca więcej)
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
    println!("  --model, -m <MODEL>         Model name to use");
    println!();
    println!("Examples:");
    println!("  acommit                                           # Use GEMINI_API_KEY env var or default Ollama");
    println!("  acommit --ollama-url http://localhost:11434       # Use local Ollama");
    println!("  acommit --model llama3.2:3b                      # Specify model");
    println!("  acommit --gemini-key xyz --model gemini-2.5-flash # Use Gemini with specific key");
    println!("  acommit -ou http://server:11434 -m codellama:7b  # Remote Ollama with CodeLlama");
}