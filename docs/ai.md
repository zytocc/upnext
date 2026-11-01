# Choosing an AI

Picks, review summaries, series guides, second chances and hidden gems use an AI. Choose it in Settings › Picks, "AI", or with `UPNEXT_LLM`:

- **`claude-cli`** (the default): `claude -p` on your Claude plan, through the [Claude Code](https://claude.ai/code) command-line tool you installed and signed in to.
- **`codex-cli`**: `codex exec` on your ChatGPT plan. UpNext turns off Codex's tools, plugins and your config for each call. The models default to the newest Luna your account has.
- **`copilot-cli`**: GitHub's [Copilot CLI](https://docs.github.com/en/copilot/concepts/agents/copilot-cli/about-copilot-cli) on your Copilot plan, Copilot Free included. Install it (`winget install GitHub.Copilot` on Windows, `npm install -g @github/copilot` anywhere) and run `copilot login` once. UpNext turns off Copilot's tools, MCP servers and custom instructions for each call. Picks default to Claude Sonnet 5.5 and summaries to Claude Haiku 5.5; on Copilot Free, choose Auto, the only model it allows.
- **`api-key`**: the Anthropic API with your key (`ANTHROPIC_API_KEY`), billed per token. `sonnet`, `opus` and `haiku` mean `claude-sonnet-5-5`, `claude-opus-5-5` and `claude-haiku-5-5`, and full model ids work too.
- **`openai-key`**: the OpenAI API with your key (`OPENAI_API_KEY`). Summaries and guides use OpenAI's cheaper Flex tier when it has room.
- **`openai`**: any OpenAI-compatible server, such as Ollama, LM Studio, llama.cpp, vLLM or OpenRouter. Give its address, model and optional key in Settings or with `UPNEXT_OPENAI_*`. A model that thinks before it answers is asked not to on your own computer or network; Settings › Picks' Thinking choice changes that.

With a CLI, UpNext runs the program on your own plan and never reads its credentials. With a local server, nothing leaves your machine.

## Models and cost

Picks and review summaries default to Sonnet on the Claude CLI. With an API key, picks default to Sonnet 5.5 and review summaries to Haiku 5.5.

Settings › AI usage shows each call's tokens and what they would cost at list prices. Review summaries, second chances and hidden gems can use a cheaper or local AI (Settings › Picks, "Summaries").

## What runs on its own

The AI jobs (review summaries, series guides, second chances, hidden gems) are off until you turn them on in Settings › Jobs or the setup wizard, which shows an estimate of what a first run takes. Picks work without an AI, from UpNext's own ranking.

`upnext prompt "…"` prints what a pick would send. [Privacy](privacy.md) lists what each AI task is given.
