<h1 align="center">Pocket Flow Project Template: Agentic Coding (Rust)</h1>

<p align="center">
  <a href="https://github.com/The-Pocket/PocketFlow" target="_blank">
    <img 
      src="./assets/banner.png" width="800"
    />
  </a>
</p>

This is a Rust project template for Agentic Coding with [Pocket Flow](https://github.com/The-Pocket/PocketFlow), a 100-line LLM framework, and your editor of choice.

## Getting Started

1. Install Rust: https://rustup.rs/
2. Set your OpenAI API key: `export OPENAI_API_KEY=your-key-here`
3. Run the project: `cargo run`

## Project Structure

- `src/main.rs` - Entry point
- `src/flow.rs` - Flow definition
- `src/nodes.rs` - Node implementations
- `src/utils.rs` - Utility functions (LLM calls, etc.)
- `Cargo.toml` - Dependencies (only pocketflow)

- We have included rules files for various AI coding assistants to help you build LLM projects:
  - [.cursorrules](.cursorrules) for Cursor AI
  - [.clinerules](.clinerules) for Cline
  - [.windsurfrules](.windsurfrules) for Windsurf
  - [.goosehints](.goosehints) for Goose
  - Configuration in [.github](.github) for GitHub Copilot
  - [CLAUDE.md](CLAUDE.md) for Claude Code
  - [GEMINI.md](GEMINI.md) for Gemini
  
- Want to learn how to build LLM projects with Agentic Coding?

  - Check out the [Agentic Coding Guidance](https://the-pocket.github.io/PocketFlow/guide.html)
    
  - Check out the [YouTube Tutorial](https://www.youtube.com/@ZacharyLLM?sub_confirmation=1)
