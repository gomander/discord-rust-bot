use std::env::var;

use serde::Deserialize;

use serenity::{all::User, model::channel::Message};

const OPENAI_API: &str = "https://api.openai.com/v1";

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
    var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    var("OPENAI_ASSISTANT_ID").expect("OPENAI_ASSISTANT_ID must be set");
    println!("OpenAI environment variables are set");
}

pub async fn create_thread(client: &reqwest::Client) -> String {
    let thread_id = client
        .post(format!("{OPENAI_API}/threads"))
        .header("Authorization", get_auth_header())
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
    println!("Created thread with ID: {thread_id}");
    thread_id
}

pub async fn add_message_to_thread(msg: &Message, thread_id: &str, client: &reqwest::Client) {
    let user_name = msg
        .author
        .global_name
        .as_deref()
        .unwrap_or(&msg.author.name);
    println!(
        "Adding message to thread {thread_id}: {user_name} ({}): \"{}\"",
        msg.author.id, msg.content
    );
    let result = client
        .post(format!("{OPENAI_API}/threads/{thread_id}/messages"))
        .header("Authorization", get_auth_header())
        .header("OpenAI-Beta", "assistants=v2")
        .json(&serde_json::json!({
            "content": format!(
                "{user_name} ({}): \"\"\"\n{}\n\"\"\"",
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
    };
}

pub async fn create_run(user: &User, thread_id: &str, client: &reqwest::Client) -> String {
    let user_name = user.global_name.as_deref().unwrap_or(&user.name);
    println!("Creating run for user: {user_name} ({})", user.id);
    let result = client
        .post(format!("{OPENAI_API}/threads/{thread_id}/runs"))
        .header("Authorization", get_auth_header())
        .header("OpenAI-Beta", "assistants=v2")
        .json(&serde_json::json!({
            "assistant_id": var("OPENAI_ASSISTANT_ID").unwrap(),
            "additional_instructions": format!(
                "The most recent message was sent by {user_name} ({})",
                user.id.to_string()
            ),
        }))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await;

    match result {
        Ok(json) => {
            let run_id = json["id"].as_str().unwrap();
            println!("Created run with ID: {run_id}");
            run_id.to_string()
        }
        Err(e) => {
            println!("Error creating run: {e:?}");
            "error".to_string()
        }
    }
}

pub async fn check_run_status(run_id: &str, thread_id: &str, client: &reqwest::Client) -> String {
    let result = client
        .get(format!("{OPENAI_API}/threads/{thread_id}/runs/{run_id}"))
        .header("Authorization", get_auth_header())
        .header("OpenAI-Beta", "assistants=v2")
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await;

    match result {
        Ok(json) => {
            let status = json["status"].as_str().unwrap();
            println!("Run status: {status}");
            status.to_string()
        }
        Err(e) => {
            println!("Error checking run status: {e:?}");
            "error".to_string()
        }
    }
}

pub async fn get_thread_run_result(
    run_id: &str,
    thread_id: &str,
    client: &reqwest::Client,
) -> String {
    let response: ThreadMessagesResponse = client
        .get(format!(
            "{OPENAI_API}/threads/{thread_id}/messages?run_id={run_id}"
        ))
        .header("Authorization", get_auth_header())
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

fn get_auth_header() -> String {
    format!("Bearer {}", var("OPENAI_API_KEY").unwrap())
}
