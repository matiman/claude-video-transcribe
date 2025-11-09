use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

/// CLI application for transcribing YouTube videos and asking questions using RAG
#[derive(Parser)]
#[command(name = "claude-video-transcribe")]
#[command(about = "Transcribe YouTube videos and ask questions using RAG with Gemini API", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch and index a YouTube video transcript
    Index {
        /// YouTube video URL
        #[arg(short, long)]
        url: String,
    },
    /// Ask a question about an indexed video
    Ask {
        /// YouTube video URL (must be indexed first)
        #[arg(short, long)]
        url: String,
        /// Question to ask about the video
        #[arg(short, long)]
        question: String,
    },
    /// Index a video and immediately ask a question
    Query {
        /// YouTube video URL
        #[arg(short, long)]
        url: String,
        /// Question to ask about the video
        #[arg(short, long)]
        question: String,
    },
}

// ===== Apify API Structures =====

#[derive(Serialize)]
struct ApifyRunInput {
    #[serde(rename = "startUrls")]
    start_urls: Vec<ApifyUrl>,
    #[serde(rename = "maxResults")]
    max_results: i32,
}

#[derive(Serialize)]
struct ApifyUrl {
    url: String,
}

#[derive(Deserialize, Debug)]
struct ApifyDatasetItem {
    text: Option<String>,
    #[serde(rename = "channelName")]
    channel_name: Option<String>,
    title: Option<String>,
}

// ===== Gemini API Structures =====

#[derive(Serialize)]
struct GeminiFile {
    file: GeminiFileData,
}

#[derive(Serialize)]
struct GeminiFileData {
    #[serde(rename = "mimeType")]
    mime_type: String,
    #[serde(rename = "displayName")]
    display_name: String,
}

#[derive(Deserialize, Debug)]
struct GeminiFileResponse {
    file: GeminiFileInfo,
}

#[derive(Deserialize, Debug)]
struct GeminiFileInfo {
    name: String,
    uri: String,
    state: String,
}

#[derive(Serialize)]
struct GeminiGenerateRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<GeminiTool>>,
}

#[derive(Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
    role: String,
}

#[derive(Serialize)]
struct GeminiPart {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_data: Option<GeminiFileDataRef>,
}

#[derive(Serialize)]
struct GeminiFileDataRef {
    file_uri: String,
    mime_type: String,
}

#[derive(Serialize)]
struct GeminiTool {
    google_search: Option<GoogleSearch>,
}

#[derive(Serialize)]
struct GoogleSearch {}

#[derive(Deserialize, Debug)]
struct GeminiGenerateResponse {
    candidates: Option<Vec<GeminiCandidate>>,
}

#[derive(Deserialize, Debug, Clone)]
struct GeminiCandidate {
    content: GeminiResponseContent,
}

#[derive(Deserialize, Debug, Clone)]
struct GeminiResponseContent {
    parts: Vec<GeminiResponsePart>,
}

#[derive(Deserialize, Debug, Clone)]
struct GeminiResponsePart {
    text: Option<String>,
}

// ===== Main Application Logic =====

struct VideoTranscriber {
    apify_api_key: String,
    gemini_api_key: String,
    client: reqwest::blocking::Client,
}

impl VideoTranscriber {
    fn new() -> Result<Self> {
        dotenv::dotenv().ok(); // Load .env file if it exists

        let apify_api_key = env::var("APIFY_API_KEY")
            .context("APIFY_API_KEY environment variable not set")?;
        let gemini_api_key = env::var("GEMINI_API_KEY")
            .context("GEMINI_API_KEY environment variable not set")?;

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(300))
            .build()?;

        Ok(Self {
            apify_api_key,
            gemini_api_key,
            client,
        })
    }

    /// Fetch transcript from YouTube using Apify YouTube Scraper
    fn fetch_transcript(&self, youtube_url: &str) -> Result<String> {
        println!("📥 Fetching transcript from YouTube using Apify...");

        // Step 1: Start the Apify actor run
        let run_input = ApifyRunInput {
            start_urls: vec![ApifyUrl {
                url: youtube_url.to_string(),
            }],
            max_results: 1,
        };

        let run_url = format!(
            "https://api.apify.com/v2/acts/streamers~youtube-scraper/runs?token={}",
            self.apify_api_key
        );

        let run_response = self
            .client
            .post(&run_url)
            .json(&run_input)
            .send()
            .context("Failed to start Apify actor run")?;

        if !run_response.status().is_success() {
            let status = run_response.status();
            let body = run_response.text().unwrap_or_default();
            anyhow::bail!("Apify run failed with status {}: {}", status, body);
        }

        let run_data: serde_json::Value = run_response
            .json()
            .context("Failed to parse Apify run response")?;

        let run_id = run_data["data"]["id"]
            .as_str()
            .context("Failed to get run ID from Apify response")?;

        println!("⏳ Waiting for Apify to process the video (run ID: {})...", run_id);

        // Step 2: Wait for the run to complete
        let mut attempts = 0;
        let max_attempts = 60; // 5 minutes max wait time
        loop {
            std::thread::sleep(Duration::from_secs(5));
            attempts += 1;

            let status_url = format!(
                "https://api.apify.com/v2/acts/streamers~youtube-scraper/runs/{}?token={}",
                run_id, self.apify_api_key
            );

            let status_response = self
                .client
                .get(&status_url)
                .send()
                .context("Failed to check Apify run status")?;

            let status_data: serde_json::Value = status_response
                .json()
                .context("Failed to parse Apify status response")?;

            let status = status_data["data"]["status"]
                .as_str()
                .context("Failed to get status from Apify response")?;

            match status {
                "SUCCEEDED" => break,
                "FAILED" | "ABORTED" | "TIMED-OUT" => {
                    anyhow::bail!("Apify run failed with status: {}", status);
                }
                _ => {
                    if attempts >= max_attempts {
                        anyhow::bail!("Apify run timed out after {} attempts", max_attempts);
                    }
                    print!(".");
                    std::io::Write::flush(&mut std::io::stdout())?;
                }
            }
        }

        println!("\n✅ Apify processing complete!");

        // Step 3: Get the dataset items
        let dataset_url = format!(
            "https://api.apify.com/v2/acts/streamers~youtube-scraper/runs/{}/dataset/items?token={}",
            run_id, self.apify_api_key
        );

        let dataset_response = self
            .client
            .get(&dataset_url)
            .send()
            .context("Failed to fetch Apify dataset")?;

        let items: Vec<ApifyDatasetItem> = dataset_response
            .json()
            .context("Failed to parse Apify dataset items")?;

        if items.is_empty() {
            anyhow::bail!("No transcript found for the video. The video might not have captions.");
        }

        let item = &items[0];
        let transcript = item
            .text
            .as_ref()
            .context("No transcript text found in the video data")?;

        if let Some(title) = &item.title {
            println!("📺 Video Title: {}", title);
        }
        if let Some(channel) = &item.channel_name {
            println!("👤 Channel: {}", channel);
        }
        println!("📝 Transcript length: {} characters", transcript.len());

        Ok(transcript.clone())
    }

    /// Upload transcript to Gemini File API
    fn upload_to_gemini(&self, transcript: &str, video_url: &str) -> Result<String> {
        println!("☁️  Uploading transcript to Gemini File API...");

        // Create a temporary file name based on the video URL
        let video_id = self.extract_video_id(video_url)?;
        let file_name = format!("youtube_transcript_{}.txt", video_id);

        // Upload file using multipart/form-data
        let upload_url = format!(
            "https://generativelanguage.googleapis.com/upload/v1beta/files?key={}",
            self.gemini_api_key
        );

        // First, create a metadata request
        let metadata = serde_json::json!({
            "file": {
                "display_name": file_name,
            }
        });

        // Use multipart upload
        let form = reqwest::blocking::multipart::Form::new()
            .text("metadata", metadata.to_string())
            .text("file", transcript.to_string());

        let upload_response = self
            .client
            .post(&upload_url)
            .header("X-Goog-Upload-Protocol", "multipart")
            .multipart(form)
            .send()
            .context("Failed to upload file to Gemini")?;

        if !upload_response.status().is_success() {
            let status = upload_response.status();
            let body = upload_response.text().unwrap_or_default();
            anyhow::bail!("Gemini file upload failed with status {}: {}", status, body);
        }

        let file_response: GeminiFileResponse = upload_response
            .json()
            .context("Failed to parse Gemini file upload response")?;

        println!("✅ File uploaded: {}", file_response.file.name);
        println!("   URI: {}", file_response.file.uri);
        println!("   State: {}", file_response.file.state);

        // Wait for file to be processed (state should be ACTIVE)
        if file_response.file.state != "ACTIVE" {
            println!("⏳ Waiting for file to be processed...");
            std::thread::sleep(Duration::from_secs(3));
        }

        Ok(file_response.file.uri)
    }

    /// Ask a question using Gemini API with the uploaded file
    fn ask_question(&self, file_uri: &str, question: &str) -> Result<String> {
        println!("🤔 Asking question: \"{}\"", question);

        let generate_url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}",
            self.gemini_api_key
        );

        let request = GeminiGenerateRequest {
            contents: vec![GeminiContent {
                parts: vec![
                    GeminiPart {
                        text: Some(format!(
                            "Based on the content of this video transcript, please answer the following question: {}\n\nProvide a detailed and accurate answer based solely on the information in the transcript.",
                            question
                        )),
                        file_data: None,
                    },
                    GeminiPart {
                        text: None,
                        file_data: Some(GeminiFileDataRef {
                            file_uri: file_uri.to_string(),
                            mime_type: "text/plain".to_string(),
                        }),
                    },
                ],
                role: "user".to_string(),
            }],
            tools: None,
        };

        let response = self
            .client
            .post(&generate_url)
            .json(&request)
            .send()
            .context("Failed to generate answer from Gemini")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            anyhow::bail!("Gemini generate failed with status {}: {}", status, body);
        }

        let generate_response: GeminiGenerateResponse = response
            .json()
            .context("Failed to parse Gemini generate response")?;

        let answer = generate_response
            .candidates
            .and_then(|candidates| candidates.first().cloned())
            .and_then(|candidate| candidate.content.parts.first().cloned())
            .and_then(|part| part.text)
            .context("No answer generated by Gemini")?;

        Ok(answer)
    }

    /// Extract video ID from YouTube URL
    fn extract_video_id(&self, url: &str) -> Result<String> {
        // Handle various YouTube URL formats
        if let Some(v_pos) = url.find("v=") {
            let id_start = v_pos + 2;
            let id_end = url[id_start..]
                .find('&')
                .map(|pos| id_start + pos)
                .unwrap_or(url.len());
            return Ok(url[id_start..id_end].to_string());
        } else if url.contains("youtu.be/") {
            if let Some(id_pos) = url.find("youtu.be/") {
                let id_start = id_pos + 9;
                let id_end = url[id_start..]
                    .find('?')
                    .map(|pos| id_start + pos)
                    .unwrap_or(url.len());
                return Ok(url[id_start..id_end].to_string());
            }
        }

        anyhow::bail!("Could not extract video ID from URL: {}", url);
    }

    /// Index a video (fetch transcript and upload to Gemini)
    fn index_video(&self, url: &str) -> Result<String> {
        let transcript = self.fetch_transcript(url)?;
        let file_uri = self.upload_to_gemini(&transcript, url)?;
        Ok(file_uri)
    }

    /// Query a video (index + ask question)
    fn query_video(&self, url: &str, question: &str) -> Result<String> {
        let file_uri = self.index_video(url)?;
        let answer = self.ask_question(&file_uri, question)?;
        Ok(answer)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let transcriber = VideoTranscriber::new()?;

    match cli.command {
        Commands::Index { url } => {
            println!("🚀 Indexing video: {}", url);
            let file_uri = transcriber.index_video(&url)?;
            println!("\n✨ Video successfully indexed!");
            println!("File URI: {}", file_uri);
            println!("\nYou can now ask questions using:");
            println!("  cargo run -- ask --url \"{}\" --question \"Your question here\"", url);
        }
        Commands::Ask { url, question } => {
            println!("🚀 Processing question for video: {}", url);
            println!("⚠️  Note: This will re-index the video. Use 'index' first for better performance.");
            let file_uri = transcriber.index_video(&url)?;
            let answer = transcriber.ask_question(&file_uri, &question)?;
            println!("\n💡 Answer:\n{}", answer);
        }
        Commands::Query { url, question } => {
            println!("🚀 Querying video: {}", url);
            let answer = transcriber.query_video(&url, &question)?;
            println!("\n💡 Answer:\n{}", answer);
        }
    }

    Ok(())
}
