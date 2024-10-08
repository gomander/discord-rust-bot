# Notes

This is just a file I'm starting to be able to save my thoughts for later.

## Dependencies

Yeah I know dotenv is unmaintained and thus not recommended, but I got tired of 98423-character-long startup commands that I had to copy and paste all the time. I figure I'll fix this if I were to ever deploy it for real.

## Discord

Discord works pretty great so far. Still want to look into creating slash commands, but last I tried it seemed difficult. The Serenity team recommends using Poise for that, but I think that might require a bit of a rewrite.

I should replace mentions in incoming messages before they're added to the thread. This should make it easier for the bot to follow conversations. Perhaps leave the mention, but add something like <#channel id> [channel name].

## OpenAI

Inactive threads are kept for 60 days before being deleted. If you don't want the bot to lose its memory, make sure to poke it every couple months.

The bot may need some internal monologue solution to be able perform complex tasks like coding, math, and picking a number without telling the user.

## Supabase

Setting up Supabase with the correct connection parameters, from URI to API keys, has been a pain. Thankfully, I've got it sorted now, and `.env.template` has all the necessary variables listed.

## Other things

Here's the NodeJS project that inspired the approach of using OpenAI assistants for memory: https://github.com/VoloBuilds/openai-assistants-discord-bot. There's a PR open with some cool feature additions that I may be able to port.

I wanna add attachment parsing, including reading text-based files, listening to audio, and vision support.

I would like to improve the logic for deciding if a message should be responded to. Constantly replying to the bot's last message is tedious. No clue how I'd accomplish that, though. Might need an extra OpenAI API call to check if it is relevant to the bot user. Maybe the internal monologue could decide?

In the same vein, I think I want to batch messages, so as not to respond to every single message all the time, especially when multiple people are talking. This might be possible by creating a new messages cache that is written to when the bot receives a message, but only acted upon periodically, then cleared.

Wondering if this whole thing where I have one thread per channel/DM is not ideal. The bot would basically be two completely different "people" in two different channels in the same server, talking to the same people. Maybe that's fine? Would be neat if it could remember your conversations no matter where they were, though. This might be doable with a runtime flag similar to debug mode that enables global memory. In that case, just hard-code the channel_id to "0" regardless of what channel the bot receives a message in.

The bot might benefit from fetching all the messages it missed while it was offline. At startup, maybe it should query the channels it has threads for and fetch all messages after the last one it has in the thread by attaching the message IDs as metadata, then adding those to the thread.
