use serenity::all::{Attachment, ChannelId, Context, GatewayIntents, User};
use std::env::var;

pub fn get_token() -> String {
	var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set!")
}

pub fn get_intents() -> GatewayIntents {
	GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES | GatewayIntents::MESSAGE_CONTENT
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
	user
		.global_name
		.as_deref()
		.unwrap_or(&user.name)
		.to_string()
}

pub fn split_message(message: &str, max_length: usize) -> Vec<&str> {
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

			// TODO: Refactor this monstrosity
			end = if in_code_block {
				// The end is currently in a code block, so we need to decide how to proceed
				if let Some(backticks_pos) = substr.rfind("```") {
					if let Some(_) =
						&message[backticks_pos + 3..(backticks_pos + max_length).min(message.len())].find("```")
					{
						// Next set of backticks are within reach of these backticks, so break before the code block starts
						in_code_block = false;
						start + backticks_pos
					} else {
						// Code block is longer than max_length, so we need to find a place to break in the code block
						if let Some(newline_pos) = substr.rfind('\n') {
							start + newline_pos + 1
						} else if let Some(semi_pos) = substr.rfind(';') {
							start + semi_pos + 1
						} else if let Some(space_pos) = substr.rfind(' ') {
							start + space_pos + 1
						} else if let Some(comma_pos) = substr.rfind(',') {
							start + comma_pos + 1
						} else {
							// This chunk will just be "```"
							start + backticks_pos + 3
						}
					}
				} else if let Some(backtick_pos) = substr.rfind('`') {
					if let Some(_) =
						&message[backtick_pos + 1..(backtick_pos + max_length).min(message.len())].find('`')
					{
						// Next backtick is within reach of this backtick, so break before the code block starts
						in_code_block = false;
						start + backtick_pos
					} else {
						// Code block is longer than max_length, so we need to find a place to break in the code block
						// For now, we just immediately break after the code block starts
						start + backtick_pos + 1
					}
				} else if let Some(pos) = substr.rfind('\n') {
					start + pos + 1
				} else if let Some(pos) = substr.rfind(';') {
					start + pos + 1
				} else if let Some(pos) = substr.rfind(' ') {
					start + pos + 1
				} else if let Some(pos) = substr.rfind(',') {
					start + pos + 1
				} else {
					end
				}
			} else {
				// The end is not in a code block
				if substr.starts_with("```") {
					// There is a code block at the start of the chunk
					if let Some(pos) = &message[start + 3..end].find("```") {
						// Terminate the chunk at the end of the code block, so that we don't split the code block unnecessarily at a newline
						start + 3 + pos + 3
					} else {
						end
					}
				} else if substr.starts_with('`') {
					if let Some(pos) = &message[start + 1..end].find('`') {
						start + 1 + pos + 1
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
				}
			};

			message[start..end].trim()
		});
		start = end;
	}

	chunks
}
