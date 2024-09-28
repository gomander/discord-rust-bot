# Discord Rust bot

This is a simple work-in-progress Discord bot that responds to messages. I'm very new to Rust, so there are bound to be oddities and inefficiencies all over the code.

## Setup

You'll need a Discord app, an OpenAI API key and an assistant, and a Supabase database.

### Discord

Create an app and add it to a server. Make sure it has all privileged gateway intents enabled and is given reasonable permissions including "View Channels", "Send Messages", and "Send Messages in Threads".

### OpenAI

Create your API key and assistant through the dashboard.

I recommend giving your assistant some preamble about their persona, followed by this.

````
You get messages in this format:
```
username (ID): """
message
"""
```
Example:
```
Bob (174673485955305072): """
hey, what's up?
"""
```
Respond with just your message; your username and ID are unnecessary.
Generally, you should refer to others by their username or a mutually agreed upon name. Do not include a leading @ when doing so. If you need to, you can mention/ping people with `<@ID>`. For example, someone can mention you with `<@1288518582475128550>`.
````

### Supabase

Create a table in Supabase called "threads_to_channels" with the primary key column being "thread_id" and another column "channel_id". Both should non-nullable text and unique.

Here is a SQL query to create such a table.

```sql
CREATE TABLE
  public.threads_to_channels (
    thread_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    CONSTRAINT threads_to_channels_pkey PRIMARY KEY (thread_id),
    CONSTRAINT threads_to_channels_channel_id_key UNIQUE (channel_id)
  ) TABLESPACE pg_default;
```

### .env file

Create a `.env` file using the template provided.

## Running the bot

Run a local dev version with `cargo run`. Run in debug mode with `DEBUG=true cargo run`.
