# Notes

This is just a file I'm starting to be able to save my thoughts for later.

## Dependencies

Yeah I know dotenv is unmaintained and thus not recommended, but I got tired of 98423-character-long startup commands that I had to copy and paste all the time. I figure I'll fix this if I were to ever deploy it for real.

## Discord

Discord works pretty great so far. Still want to look into creating slash commands, but last I tried it seemed difficult. The Serenity team recommends using Poise for that, but I think that might require a bit of a rewrite.

## OpenAI

Inactive threads are kept for 60 days before being deleted. If you don't want the bot to lose its memory, make sure to poke it every couple months.

## Supabase

Setting up Supabase with the correct connection parameters, from URI to API keys, has been a pain. Thankfully, I've got it sorted now, and `.env.template` has all the necessary variables listed.

## Other things

Here's the NodeJS project that inspired the approach of using OpenAI assistants for memory: https://github.com/VoloBuilds/openai-assistants-discord-bot. There's a PR open with some cool feature additions that I may be able to port.

I wanna add attachment parsing, including reading text-based files, listening to audio, and vision support.

I would like to improve the logic for deciding if a message should be responded to. Constantly replying to the bot's last message is tedious.

In the same vein, I think I want to batch messages, so as not to respond to every single message all the time, especially when multiple people are talking.

Wondering if this whole thing where I have one thread per channel/DM is not ideal. The bot would basically be two completely different "people" in two different channels in the same server, talking to the same people. Maybe that's fine? Would be neat if it could remember your conversations no matter where they were, though.
