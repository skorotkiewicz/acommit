use std::env;
use std::process::Command;
use std::io::{self, Write};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
}

#[derive(Deserialize)]
struct Candidate {
    content: Option<ResponseContent>,
}

#[derive(Deserialize)]
struct ResponseContent {
    parts: Option<Vec<ResponsePart>>,
}

#[derive(Deserialize)]
struct ResponsePart {
    text: Option<String>,
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Pobierz klucz API z zmiennej środowiskowej
    let api_key = env::var("GEMINI_API_KEY")
        .map_err(|_| "GEMINI_API_KEY environment variable not set")?;

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

    // Wywołaj Gemini API
    let commit_message = call_gemini_api(&api_key, &prompt).await?;
    
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

async fn call_gemini_api(api_key: &str, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let model = "gemini-2.5-flash-lite";
    
    let request_body = GeminiRequest {
        contents: vec![Content {
            parts: vec![Part {
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
        return Err(format!("API request failed: {}", response.status()).into());
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
