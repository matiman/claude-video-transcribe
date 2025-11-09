# claude-video-transcribe

Write a Rust CLI application that accepts any YouTube video link, transcribes the content, and utilizes the Gemini File Search API (see https://blog.google/technology/developers/file-search-gemini-api/) to build a Retrieval-Augmented Generation (RAG) system for that video. The CLI should then allow you to ask questions about the video, with answers based on the RAG pipeline.

## Task Details
- The CLI must:
    1. Accept a YouTube video URL as user input.
    2. Download the audio from the specified YouTube video.
    3. Transcribe the audio to text.
    4. Use the transcript to create a knowledge base and integrate with the Gemini API for RAG.
    5. Enable interactive Q&A: accept user questions and answer them according to the info indexed from the video.
- The prompt should instruct the AI to:
    - Plan the overall architecture before coding.
    - Recommend third-party crates (for YouTube download, transcription, Gemini API integration, CLI parsing, etc.).
    - Write a single well-commented, production-grade Rust file or describe clear module separation (if needed).
    - Ensure input validation, error handling, and API-key management are included.
    - Reasoning/planning must come before any code in the output.

## Output Format
- The AI’s output must have:
    - Step-by-step reasoning/plan (1-2 paragraphs in markdown).
    - The Rust source code (main.rs) with comments, separated by clear markdown section headings.
    - No code blocks—output as plain text with markdown.
    - If the program is large, break the code into logical, explained sections/modules.

## Example

### Example Input
Write a Rust CLI that can take a YouTube URL, transcribe its contents, index with Gemini File Search API for RAG, and let me ask questions about it.

### Example Output

#### Reasoning / Plan
*(A concise, high-level architectural overview and design explanation goes here.)*

#### Complete main.rs Source
*(Full, well-commented Rust code appears here, segmented with explanations for each module or section if needed.)*

---

**Important:**  
- Always show the reasoning and planning section before the code.
- The output should use markdown headings for "Reasoning / Plan" and "Source Code".
- Make sure all steps—download, transcribe, index/upload, query, error handling—are included.
- Use placeholders for any necessary API credentials or endpoints.
