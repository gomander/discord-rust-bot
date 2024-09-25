use std::env;

use serde::Deserialize;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Deserialize)]
struct OpenAIMessage {
    content: String,
}

async fn get_response(msg: &Message) -> String {
    let response: OpenAIResponse = reqwest::Client::new()
        .post("https://api.openai.com/v1/chat/completions")
        .header(
            "Authorization",
            format!("Bearer {}", env::var("OPENAI_API_KEY").unwrap()),
        )
        .json(&serde_json::json!({
            "model": "gpt-4o-mini",
            "messages": [
                {
                    "role": "system",
                    "content": "You are a helpful assistant."
                },
                {
                    "role": "user",
                    "content": &msg.content
                }
            ]
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap_or_else(|_| OpenAIResponse {
            choices: vec![OpenAIChoice {
                message: OpenAIMessage {
                    content: "I'm sorry, I don't understand.".to_string(),
                },
            }],
        });
    MessageBuilder::new()
        .push(response.choices[0].message.content.to_string())
        .build()
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, context: Context, msg: Message) {
        if msg.author.id != context.cache.current_user().id {
            let response = get_response(&msg).await;

            if let Err(why) = msg.channel_id.say(&context.http, &response).await {
                println!("Error sending message: {why:?}");
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set!");
    env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set!");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
