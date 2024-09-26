# Notes

This is just a file I'm starting to be able to save my thoughts for later.

## Dependencies

Yeah I know dotenv is unmaintained and thus not recommended, but I got tired of 9842375023948 character-long startup commands that I had to copy and paste all the time. I figure I'll fix this if I were to ever deploy it for real.

## Discord

Discord works pretty great so far. Still want to look into creating slash commands, but last I tried it seemed difficult. The Serenity team recommends using Poise for that, but I think that might require a bit of a rewrite.

I think it'd be neat if I could do some presence stuff, like setting the bot's status and telling the channel that the bot is "typing" while it's cooking a response.

## OpenAI

Inactive threads are kept for 60 days before being deleted. If you don't want the bot to lose its memory, make sure to poke it every couple months.

## Supabase

WOW getting the Supabase integration working was harder than it needed to be. The documentation is bad. It's so hard to find exactly the article I need, and odds are it doesn't have complete information anyway. I've been forced to disable RLS on my table because I couldn't write to it otherwise, even using my service key. Whatever, the credentials are only going to be stored on the bot server anyway.

## Other things

I wanna add attachment parsing, including reading text-based files, listening to audio, and vision support.

I would like to improve the logic for deciding if a message should be responded to. Constantly replying is tedious.

Wondering if this whole thing where I have one thread per channel/DM is not ideal. The bot would basically be two completely different "people" in two different channels in the same server, talking to the same people. Maybe that's fine? Would be neat if it could remember your conversations no matter where they were, though.
