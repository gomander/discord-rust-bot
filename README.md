# Discord Rust bot

This is a simple work-in-progress Discord bot that responds to messages. I'm very new to Rust, so there are bound to be oddities and inefficiencies all over the code.

## Setup

You'll need an OpenAI API key, an OpenAI assistant, and an OpenAI thread, as well as a Discord bot token.

### Discord

Create an app and add it to a server. Make sure it has all intents enabled and is given reasonable permissions.

### OpenAI

Create your API key and assistant through the dashboard, then create a thread by hitting the endpoint manually. This is something I intend to automate away eventually.

## Running the bot

```sh
DISCORD_TOKEN=AAAAA OPENAI_API_KEY=sk-proj-AAAAA OPENAI_ASSISTANT_ID=asst_AAAAA OPENAI_THREAD_ID=thread_AAAAA cargo run
```
