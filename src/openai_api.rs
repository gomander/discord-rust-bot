use std::env;

use serde::Deserialize;

use serenity::{all::User, model::channel::Message};

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

pub async fn create_thread(client: &reqwest::Client) -> String {
    client
        .post("https://api.openai.com/v1/threads")
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
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string()
}

pub async fn add_message_to_thread(msg: &Message, client: &reqwest::Client) {
    let user_name = msg
        .author
        .global_name
        .as_deref()
        .unwrap_or(&msg.author.name);
    client
        .post(format!(
            "https://api.openai.com/v1/threads/{}/messages",
            env::var("OPENAI_THREAD_ID").unwrap()
        ))
        .header(
            "Authorization",
            format!("Bearer {}", env::var("OPENAI_API_KEY").unwrap()),
        )
        .header("OpenAI-Beta", "assistants=v2")
        .json(&serde_json::json!({
            "content": format!(
                "{} ({}): \"\"\"\n{}\n\"\"\"",
                user_name,
                msg.author.id.to_string(),
                msg.content
            ),
            "role": "user",
        }))
        .send()
        .await
        .unwrap();
}

pub async fn create_run(user: &User, client: &reqwest::Client) -> String {
    let user_name = user
        .global_name
        .as_deref()
        .unwrap_or(&user.name);
    client
        .post(format!(
            "https://api.openai.com/v1/threads/{}/runs",
            env::var("OPENAI_THREAD_ID").unwrap()
        ))
        .header(
            "Authorization",
            format!("Bearer {}", env::var("OPENAI_API_KEY").unwrap()),
        )
        .header("OpenAI-Beta", "assistants=v2")
        .json(&serde_json::json!({
            "assistant_id": env::var("OPENAI_ASSISTANT_ID").unwrap(),
            "additional_instructions": format!(
                "The most recent message was sent by {} ({})",
                user_name,
                user.id.to_string()
            ),
        }))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string()
}

pub async fn check_run_status(run_id: &str, client: &reqwest::Client) -> String {
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
        .as_str()
        .unwrap()
        .to_string()
}

pub async fn get_thread_run_result(run_id: &str, client: &reqwest::Client) -> String {
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
