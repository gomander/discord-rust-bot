use std::env;

use serde::Deserialize;
use serenity::model::id::ChannelId;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

#[derive(Deserialize)]
struct ThreadMessagesResponse {
    data: Vec<ThreadMessage>,
}

#[derive(Deserialize)]
struct ThreadMessage {
    content: Vec<ThreadMessageContent>,
}

#[derive(Deserialize)]
struct ThreadMessageContent {
    text: ThreadMessageContentText,
}

#[derive(Deserialize)]
struct ThreadMessageContentText {
    value: String,
}

async fn add_message_to_thread(msg: &Message, client: &reqwest::Client) {
    client
        .post(&format!(
            "https://api.openai.com/v1/threads/{}/messages",
            env::var("OPENAI_THREAD_ID").unwrap()
        ))
        .header(
            "Authorization",
            format!("Bearer {}", env::var("OPENAI_API_KEY").unwrap()),
        )
        .header("OpenAI-Beta", "assistants=v2")
        .json(&serde_json::json!({
            "content": msg.content,
            "role": "user"
        }))
        .send()
        .await
        .unwrap();
}

async fn create_run(client: &reqwest::Client) -> String {
    client
        .post(&format!(
            "https://api.openai.com/v1/threads/{}/runs",
            env::var("OPENAI_THREAD_ID").unwrap()
        ))
        .header(
            "Authorization",
            format!("Bearer {}", env::var("OPENAI_API_KEY").unwrap()),
        )
        .header("OpenAI-Beta", "assistants=v2")
        .json(&serde_json::json!({
            "assistant_id": env::var("OPENAI_ASSISTANT_ID").unwrap()
        }))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .to_string()
        .trim_matches('"')
        .to_string()
}

async fn check_run_status(run_id: &str, client: &reqwest::Client) -> String {
    client
        .get(format!(
            "https://api.openai.com/v1/threads/{}/runs/{}",
            env::var("OPENAI_THREAD_ID").unwrap(),
            run_id
        ))
        .header(
            "Authorization",
            format!("Bearer {}", env::var("OPENAI_API_KEY").unwrap()),
        )
        .header("OpenAI-Beta", "assistants=v2")
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap()["status"]
        .to_string()
        .trim_matches('"')
        .to_string()
    }

async fn get_response(msg: &Message) -> String {
    let client = reqwest::Client::new();

    add_message_to_thread(&msg, &client).await;

    let run_id = create_run(&client).await;

    let terminal_statuses = ["completed", "expired", "failed", "cancelled", "incomplete"];

    for _ in 0..20 {
        let status = check_run_status(&run_id, &client).await;
        if terminal_statuses.contains(&status.as_str()) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(3));
    }

    let response: ThreadMessagesResponse = client
        .get(format!(
            "https://api.openai.com/v1/threads/{}/messages?run_id={}",
            env::var("OPENAI_THREAD_ID").unwrap(),
            run_id
        ))
        .header(
            "Authorization",
            format!("Bearer {}", env::var("OPENAI_API_KEY").unwrap()),
        )
        .header("OpenAI-Beta", "assistants=v2")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    if response.data.is_empty() || response.data[0].content.is_empty() {
        return "No response from OpenAI".to_string();
    }

    response.data[0].content[0].text.value.to_string()
}

async fn send_response(response: &str, channel_id: &ChannelId, context: &Context) {
    for chunk in response.chars().collect::<Vec<_>>().chunks(1000) {
        let message = MessageBuilder::new()
            .push(chunk.iter().collect::<String>())
            .build();

        if let Err(why) = channel_id.say(&context.http, &message).await {
            println!("Error sending message: {why:?}");
        }
    }
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, context: Context, msg: Message) {
        if !msg.author.bot {
            let response = get_response(&msg).await;
            send_response(&response, &msg.channel_id, &context).await;
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
    env::var("OPENAI_ASSISTANT_ID").expect("OPENAI_ASSISTANT_ID not set!");
    env::var("OPENAI_THREAD_ID").expect("OPENAI_THREAD_ID not set!");
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
