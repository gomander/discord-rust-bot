use serenity::all::{Attachment, ChannelId, Context, GatewayIntents, MessageBuilder, User};
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
    for chunk in message.chars().collect::<Vec<_>>().chunks(2000) {
        let message = MessageBuilder::new()
            .push(chunk.iter().collect::<String>())
            .build();

        if let Err(e) = channel_id.say(&context.http, &message).await {
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
