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
}

pub async fn create_thread(client: &reqwest::Client) -> String {
    let result = client
        .post(format!("{OPENAI_API}/threads"))
        .header("Authorization", get_auth_header())
        .header("OpenAI-Beta", "assistants=v2")
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await;

    match result {
        Ok(json) => json["id"].as_str().unwrap().to_string(),
        Err(e) => {
            println!("Error creating thread: {e:?}");
            "error".to_string()
        }
    }
}

pub async fn add_message_to_thread(msg: &Message, thread_id: &str, client: &reqwest::Client) {
    let user_name = get_user_name(&msg.author);
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
    };
}

pub async fn create_run(user: &User, thread_id: &str, client: &reqwest::Client) -> String {
    let user_name = get_user_name(user);
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

fn get_user_name(user: &User) -> String {
    user.global_name
        .as_deref()
        .unwrap_or(&user.name)
        .to_string()
}
