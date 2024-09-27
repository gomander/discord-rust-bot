# Discord Rust bot

This is a simple work-in-progress Discord bot that responds to messages. I'm very new to Rust, so there are bound to be oddities and inefficiencies all over the code.

## Setup

You'll need a Discord app, an OpenAI API key and an assistant, and a Supabase database.

### Discord

Create an app and add it to a server. Make sure it has all intents enabled and is given reasonable permissions.

### OpenAI

Create your API key and assistant through the dashboard, then create a thread by hitting the endpoint manually. This is something I intend to automate away eventually.

### Supabase

Create a table in Supabase called "threads_to_channels" with the primary key column being "thread_id" and another column "channel_id". Both should be unique and text type.

### .env file

Create a `.env` file using the template provided.

## Running the bot

Run a local dev version with `cargo run`. Run in debug mode with `DEBUG=true cargo run`.
