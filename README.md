# claude-video-transcribe

A Rust CLI application that transcribes YouTube videos and enables intelligent Q&A using RAG (Retrieval-Augmented Generation) powered by the Gemini API.

## Features

- 📥 **Automatic Transcript Extraction**: Fetches YouTube video transcripts using Apify YouTube Scraper
- ☁️ **Gemini Integration**: Uploads transcripts to Gemini File API for advanced RAG capabilities
- 🤔 **Intelligent Q&A**: Ask questions about video content and get accurate answers based on the transcript
- 🚀 **Fast & Efficient**: No need to download video/audio files - works directly with YouTube captions
- 🔒 **Secure**: API keys managed via environment variables

## Architecture

This CLI application follows a simple but powerful architecture:

1. **Transcript Fetching**: Uses Apify YouTube Scraper API to extract video transcripts directly from YouTube (leveraging available captions/subtitles)
2. **File Upload**: Uploads the transcript to Gemini File API for indexing
3. **RAG Query**: Uses Gemini's generative capabilities with the uploaded file as context to answer questions

### Why This Approach?

- **Fast**: No audio download or local transcription needed
- **Cost-effective**: Leverages existing YouTube captions instead of expensive transcription services
- **Accurate**: Uses Google's Gemini for high-quality answer generation

## Prerequisites

- Rust (latest stable version)
- Apify API key (get one at https://console.apify.com/account/integrations)
- Gemini API key (get one at https://makersuite.google.com/app/apikey)

## Installation

1. Clone the repository:
```bash
git clone https://github.com/matiman/claude-video-transcribe.git
cd claude-video-transcribe
```

2. Set up environment variables:
```bash
cp .env.example .env
# Edit .env and add your API keys
```

3. Build the project:
```bash
cargo build --release
```

## Configuration

Create a `.env` file in the project root with your API keys:

```env
APIFY_API_KEY=your_apify_api_key_here
GEMINI_API_KEY=your_gemini_api_key_here
```

**Important**: Never commit your `.env` file to version control. It's already in `.gitignore`.

## Usage

The CLI provides three main commands:

### 1. Index a Video

Fetch and index a YouTube video transcript:

```bash
cargo run -- index --url "https://www.youtube.com/watch?v=VIDEO_ID"
```

This will:
- Fetch the transcript from YouTube
- Upload it to Gemini File API
- Print the file URI for reference

### 2. Ask a Question (with re-indexing)

Ask a question about a video (will re-index the video):

```bash
cargo run -- ask --url "https://www.youtube.com/watch?v=VIDEO_ID" --question "What is the main topic?"
```

### 3. Query (Index + Ask in one command)

Index a video and immediately ask a question:

```bash
cargo run -- query --url "https://www.youtube.com/watch?v=VIDEO_ID" --question "What are the key takeaways?"
```

### Examples

```bash
# Index a video about Rust programming
cargo run -- index --url "https://www.youtube.com/watch?v=BpPEoZW5IiY"

# Ask about the video content
cargo run -- ask --url "https://www.youtube.com/watch?v=BpPEoZW5IiY" --question "What are the main features of Rust?"

# Query in one step
cargo run -- query --url "https://www.youtube.com/watch?v=BpPEoZW5IiY" --question "What memory safety features does Rust have?"
```

### Help

Get help on available commands:

```bash
cargo run -- --help
cargo run -- index --help
cargo run -- ask --help
cargo run -- query --help
```

## How It Works

1. **Transcript Extraction**:
   - The CLI uses Apify's YouTube Scraper actor to extract transcripts
   - It waits for the scraping job to complete (usually takes 5-30 seconds)
   - Retrieves the video title, channel name, and full transcript text

2. **Gemini Upload**:
   - The transcript is uploaded to Gemini File API as a text file
   - Gemini processes and indexes the content for retrieval

3. **Question Answering**:
   - When you ask a question, it's sent to Gemini along with a reference to the uploaded file
   - Gemini uses RAG to find relevant information in the transcript
   - Returns a detailed, context-aware answer

## Dependencies

- `clap`: CLI argument parsing
- `reqwest`: HTTP client for API calls
- `serde`/`serde_json`: JSON serialization
- `dotenv`: Environment variable management
- `anyhow`: Error handling
- `tokio`: Async runtime

## Limitations

- Only works with YouTube videos that have captions/subtitles available
- Transcript quality depends on YouTube's automatic or manual captions
- Apify free tier has usage limits
- Gemini API has rate limits and quotas

## Error Handling

The application includes comprehensive error handling for:
- Missing API keys
- Invalid YouTube URLs
- Videos without captions
- API failures and timeouts
- Network issues

## Security

- API keys are managed via environment variables
- `.env` file is excluded from version control
- No sensitive data is logged or stored

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - feel free to use this project for any purpose.

## Troubleshooting

**Video has no transcript**:
- Make sure the video has captions/subtitles available
- Try a different video

**Apify timeout**:
- The video might be very long
- Check your Apify account quota

**Gemini API errors**:
- Verify your API key is correct
- Check you haven't exceeded rate limits
- Ensure you have the Gemini API enabled in Google Cloud Console

## Roadmap

- [ ] Support for multiple video formats
- [ ] Local caching of transcripts
- [ ] Batch processing of multiple videos
- [ ] Export Q&A sessions to markdown
- [ ] Support for other video platforms
