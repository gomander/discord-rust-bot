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

pub fn verify_env_vars() {
    env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    env::var("OPENAI_ASSISTANT_ID").expect("OPENAI_ASSISTANT_ID must be set");
    println!("OpenAI environment variables are set");
}

pub async fn create_thread(client: &reqwest::Client) -> String {
    let thread_id = client
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
        .to_string();
    println!("Created thread with ID: {}", thread_id);
    thread_id
}

pub async fn add_message_to_thread(msg: &Message, thread_id: &str, client: &reqwest::Client) {
    let user_name = msg
        .author
        .global_name
        .as_deref()
        .unwrap_or(&msg.author.name);
    println!(
        "Adding message to thread {}: {} ({}): \"{}\"",
        thread_id, user_name, msg.author.id, msg.content
    );
    let result = client
        .post(format!(
            "https://api.openai.com/v1/threads/{}/messages",
            thread_id
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
        .await;
    if let Err(e) = result {
        println!("Error adding message to thread: {:?}", e);
    } else {
        println!("Added message to thread");
    }
}

pub async fn create_run(user: &User, thread_id: &str, client: &reqwest::Client) -> String {
    let user_name = user.global_name.as_deref().unwrap_or(&user.name);
    println!("Creating run for user: {} ({})", user_name, user.id);
    client
        .post(format!(
            "https://api.openai.com/v1/threads/{}/runs",
            thread_id
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

pub async fn check_run_status(run_id: &str, thread_id: &str, client: &reqwest::Client) -> String {
    client
        .get(format!(
            "https://api.openai.com/v1/threads/{}/runs/{}",
            thread_id, run_id
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

pub async fn get_thread_run_result(
    run_id: &str,
    thread_id: &str,
    client: &reqwest::Client,
) -> String {
    let response: ThreadMessagesResponse = client
        .get(format!(
            "https://api.openai.com/v1/threads/{}/messages?run_id={}",
            thread_id, run_id
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
