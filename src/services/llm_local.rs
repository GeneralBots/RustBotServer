use actix_web::{post, web, HttpRequest, HttpResponse, Result};
use dotenv::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;
use tokio::time::{sleep, Duration};

// Global process handle
static mut LLAMA_PROCESS: Option<Arc<Mutex<Option<tokio::process::Child>>>> = None;

// OpenAI-compatible request/response structures
#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatCompletionResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Choice {
    message: ChatMessage,
    finish_reason: String,
}

// Llama.cpp server request/response structures
#[derive(Debug, Serialize, Deserialize)]
struct LlamaCppRequest {
    prompt: String,
    n_predict: Option<i32>,
    temperature: Option<f32>,
    top_k: Option<i32>,
    top_p: Option<f32>,
    stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LlamaCppResponse {
    content: String,
    stop: bool,
    generation_settings: Option<serde_json::Value>,
}

// Function to check if server is running
async fn is_server_running(url: &str) -> bool {
    let client = Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();

    match client.get(&format!("{}/health", url)).send().await {
        Ok(response) => {
            let is_ok = response.status().is_success();
            if is_ok {
                println!("🟢 Server health check: OK");
            } else {
                println!(
                    "🔴 Server health check: Failed with status {}",
                    response.status()
                );
            }
            is_ok
        }
        Err(e) => {
            println!("🔴 Server health check: Connection failed - {}", e);
            false
        }
    }
}

// Function to start llama.cpp server
async fn start_llama_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🚀 Starting llama.cpp server...");

    // Get environment variables for llama.cpp configuration
    let llama_path = env::var("LLM_CPP_PATH").unwrap_or_else(|_| "llama-server".to_string());
    let model_path = env::var("LLM_MODEL_PATH")
        .unwrap_or_else(|_| "./models/tinyllama-1.1b-q4_01.gguf".to_string());
    let cpu_limit = env::var("CPU_LIMIT").unwrap_or_else(|_| "50".to_string());
    let port = env::var("LLM_PORT").unwrap_or_else(|_| "8080".to_string());

    println!("🔧 Configuration:");
    println!("   - Llama path: {}", llama_path);
    println!("   - Model path: {}", model_path);
    println!("   - CPU limit: {}%", cpu_limit);
    println!("   - Port: {}", port);

    // Kill any existing llama processes
    println!("🧹 Cleaning up existing processes...");
    let _ = Command::new("pkill").arg("-f").arg("llama-server").output();

    // Wait a bit for cleanup
    sleep(Duration::from_secs(2)).await;

    // Build the command
    let full_command = format!(
        "cpulimit -l {} -- {} -m '{}' --n-gpu-layers 18 --temp 0.7 --ctx-size 1024 --batch-size 256 --no-mmap --mlock --port {} --host 127.0.0.1 --tensor-split 1.0 --main-gpu 0",
        cpu_limit, llama_path, model_path, port
    );

    println!("📝 Executing command: {}", full_command);

    // Start llama.cpp server with cpulimit using tokio
    let mut cmd = TokioCommand::new("sh");
    cmd.arg("-c");
    cmd.arg(&full_command);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.kill_on_drop(true);

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to start llama.cpp server: {}", e))?;

    println!("🔄 Process spawned with PID: {:?}", child.id());

    // Capture stdout and stderr for real-time logging
    if let Some(stdout) = child.stdout.take() {
        let stdout_reader = BufReader::new(stdout);
        tokio::spawn(async move {
            let mut lines = stdout_reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                println!("🦙📤 STDOUT: {}", line);
            }
            println!("🦙📤 STDOUT stream ended");
        });
    }

    if let Some(stderr) = child.stderr.take() {
        let stderr_reader = BufReader::new(stderr);
        tokio::spawn(async move {
            let mut lines = stderr_reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                println!("🦙📥 STDERR: {}", line);
            }
            println!("🦙📥 STDERR stream ended");
        });
    }

    // Store the process handle
    unsafe {
        LLAMA_PROCESS = Some(Arc::new(Mutex::new(Some(child))));
    }

    println!("✅ Llama.cpp server process started!");
    Ok(())
}

// Function to ensure llama.cpp server is running
pub async fn ensure_llama_server_running() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let llama_url = env::var("LLM_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    // Check if server is already running
    if is_server_running(&llama_url).await {
        println!("✅ Llama.cpp server is already running");
        return Ok(());
    }

    // Start the server
    start_llama_server().await?;

    // Wait for server to be ready with verbose logging
    println!("⏳ Waiting for llama.cpp server to become ready...");
    let mut attempts = 0;
    let max_attempts = 60; // 2 minutes total

    while attempts < max_attempts {
        sleep(Duration::from_secs(2)).await;

        print!(
            "🔍 Checking server health (attempt {}/{})... ",
            attempts + 1,
            max_attempts
        );

        if is_server_running(&llama_url).await {
            println!("✅ SUCCESS!");
            println!("🎉 Llama.cpp server is ready and responding!");
            return Ok(());
        } else {
            println!("❌ Not ready yet");
        }

        attempts += 1;
        if attempts % 10 == 0 {
            println!(
                "⏰ Still waiting for llama.cpp server... (attempt {}/{})",
                attempts, max_attempts
            );
            println!("💡 Check the logs above for any errors from the llama server");
        }
    }

    Err("❌ Llama.cpp server failed to start within timeout (2 minutes)".into())
}

// Convert OpenAI chat messages to a single prompt
fn messages_to_prompt(messages: &[ChatMessage]) -> String {
    let mut prompt = String::new();

    for message in messages {
        match message.role.as_str() {
            "system" => {
                prompt.push_str(&format!("System: {}\n\n", message.content));
            }
            "user" => {
                prompt.push_str(&format!("User: {}\n\n", message.content));
            }
            "assistant" => {
                prompt.push_str(&format!("Assistant: {}\n\n", message.content));
            }
            _ => {
                prompt.push_str(&format!("{}: {}\n\n", message.role, message.content));
            }
        }
    }

    prompt.push_str("Assistant: ");
    prompt
}

// Cleanup function
pub fn cleanup_processes() {
    println!("🧹 Cleaning up llama.cpp processes...");

    unsafe {
        if let Some(process_handle) = &LLAMA_PROCESS {
            if let Ok(mut process) = process_handle.lock() {
                if let Some(ref mut child) = *process {
                    println!("🔪 Killing llama server process...");
                    let _ = child.start_kill();
                }
            }
        }
    }

    // Kill any remaining llama processes
    println!("🔍 Killing any remaining llama-server processes...");
    let output = Command::new("pkill").arg("-f").arg("llama-server").output();

    match output {
        Ok(result) => {
            if result.status.success() {
                println!("✅ Successfully killed llama processes");
            } else {
                println!("ℹ️ No llama processes found to kill");
            }
        }
        Err(e) => println!("⚠️ Error trying to kill processes: {}", e),
    }
}

// Proxy endpoint
#[post("/v1/chat/completions")]
pub async fn chat_completions(
    req_body: web::Json<ChatCompletionRequest>,
    _req: HttpRequest,
) -> Result<HttpResponse> {
    dotenv().ok();

    // Ensure llama.cpp server is running
    if let Err(e) = ensure_llama_server_running().await {
        eprintln!("Failed to start llama.cpp server: {}", e);
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": {
                "message": format!("Failed to start llama.cpp server: {}", e),
                "type": "server_error"
            }
        })));
    }

    // Get llama.cpp server URL
    let llama_url = env::var("LLM_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    // Convert OpenAI format to llama.cpp format
    let prompt = messages_to_prompt(&req_body.messages);

    let llama_request = LlamaCppRequest {
        prompt,
        n_predict: Some(500), // Adjust as needed
        temperature: Some(0.7),
        top_k: Some(40),
        top_p: Some(0.9),
        stream: req_body.stream,
    };

    // Send request to llama.cpp server
    let client = Client::builder()
        .timeout(Duration::from_secs(120)) // 2 minute timeout
        .build()
        .map_err(|e| {
            eprintln!("Error creating HTTP client: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to create HTTP client")
        })?;

    let response = client
        .post(&format!("{}/completion", llama_url))
        .header("Content-Type", "application/json")
        .json(&llama_request)
        .send()
        .await
        .map_err(|e| {
            eprintln!("Error calling llama.cpp server: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to call llama.cpp server")
        })?;

    let status = response.status();

    if status.is_success() {
        let llama_response: LlamaCppResponse = response.json().await.map_err(|e| {
            eprintln!("Error parsing llama.cpp response: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to parse llama.cpp response")
        })?;

        // Convert llama.cpp response to OpenAI format
        let openai_response = ChatCompletionResponse {
            id: format!("chatcmpl-{}", uuid::Uuid::new_v4()),
            object: "chat.completion".to_string(),
            created: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            model: req_body.model.clone(),
            choices: vec![Choice {
                message: ChatMessage {
                    role: "assistant".to_string(),
                    content: llama_response.content.trim().to_string(),
                },
                finish_reason: if llama_response.stop {
                    "stop".to_string()
                } else {
                    "length".to_string()
                },
            }],
        };

        Ok(HttpResponse::Ok().json(openai_response))
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());

        eprintln!("Llama.cpp server error ({}): {}", status, error_text);

        let actix_status = actix_web::http::StatusCode::from_u16(status.as_u16())
            .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);

        Ok(HttpResponse::build(actix_status).json(serde_json::json!({
            "error": {
                "message": error_text,
                "type": "server_error"
            }
        })))
    }
}

// Health check endpoint
#[actix_web::get("/health")]
pub async fn health() -> Result<HttpResponse> {
    let llama_url = env::var("LLM_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    if is_server_running(&llama_url).await {
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "healthy",
            "llama_server": "running"
        })))
    } else {
        Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "unhealthy",
            "llama_server": "not running"
        })))
    }
}
