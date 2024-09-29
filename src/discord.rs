use serenity::all::{Attachment, ChannelId, Context, GatewayIntents, User};
use std::env::var;

pub fn get_token() -> String {
    var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set!")
}

pub fn get_intents() -> GatewayIntents {
    GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
}

pub async fn send_message(message: &str, channel_id: &ChannelId, context: &Context) {
    for chunk in split_message(message, 2000) {
        if let Err(e) = channel_id.say(&context.http, chunk).await {
            println!("Error sending message: {e:#?}");
        }
    }
}

pub async fn get_attachment_data(attachment: &Attachment, debug: bool) -> Option<String> {
    if debug {
        println!("Attachment: {:#?}", attachment.filename);
    }

    match attachment.download().await {
        Ok(data) => match String::from_utf8(data) {
            Ok(decoded) => Some(decoded),
            Err(e) => {
                println!("Error decoding attachment: {e:#?}");
                None
            }
        },
        Err(e) => {
            println!("Error downloading attachment: {e:#?}");
            None
        }
    }
}

pub fn get_user_name(user: &User) -> String {
    user.global_name
        .as_deref()
        .unwrap_or(&user.name)
        .to_string()
}

fn split_message(message: &str, max_length: usize) -> Vec<&str> {
    if message.len() <= max_length {
        return vec![&message];
    };

    let mut chunks = Vec::new();
    let mut start = 0;
    let mut end;
    let mut in_code_block = false;

    while start < message.len() {
        end = (start + max_length).min(message.len());

        let substr = &message[start..end];

        chunks.push(if end == message.len() {
            substr.trim()
        } else {
            for (_, c) in substr.char_indices() {
                if c == '`' {
                    in_code_block = !in_code_block;
                };
            }

            // TODO: Handle code blocks better
            // The problem now is that if a code block is split between two chunks, the starting
            // backticks will terminate the current chunk when they should be included at the start
            // of the next chunk.
            // Unfortunately, simply removing the + 3 or + 1 from the end calculation will cause
            // an infinte loop of adding 0 length strs to the chunks vector if the code block
            // is longer than max_length, since the rfind will always find the starting backticks.
            // This entire block needs to be rethought.

            // I added a check to see if the next set of backticks is within the next max_length, in
            // which case I terminate the chunk before the starting backticks.

            // I am so sorry.
            end = if in_code_block {
                if let Some(pos) = substr.rfind("```") {
                    in_code_block = false;
                    if let Some(_) =
                        &message[pos + 3..(pos + max_length).min(message.len())].rfind("```")
                    {
                        start + pos
                    } else {
                        start + pos + 3
                    }
                } else if let Some(pos) = substr.rfind('`') {
                    in_code_block = false;
                    if let Some(_) =
                        &message[pos + 1..(pos + max_length).min(message.len())].rfind('`')
                    {
                        start + pos
                    } else {
                        start + pos + 1
                    }
                } else if let Some(pos) = substr.rfind('\n') {
                    start + pos + 1
                } else if let Some(pos) = substr.rfind(' ') {
                    start + pos + 1
                } else if let Some(pos) = substr.rfind(',') {
                    start + pos + 1
                } else {
                    end
                }
            } else {
                if let Some(pos) = substr.rfind('\n') {
                    start + pos + 1
                } else if let Some(pos) = substr.rfind(". ") {
                    start + pos + 2
                } else if let Some(pos) = substr.rfind(' ') {
                    start + pos + 1
                } else if let Some(pos) = substr.rfind('.') {
                    start + pos + 1
                } else if let Some(pos) = substr.rfind(',') {
                    start + pos + 1
                } else {
                    end
                }
            };

            message[start..end].trim()
        });
        start = end;
    }

    chunks
}
