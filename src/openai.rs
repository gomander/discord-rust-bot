use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::env::var;

const OPENAI_API: &str = "https://api.openai.com/v1";

#[derive(Deserialize)]
struct GetThreadMessagesResponse {
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

#[derive(Deserialize)]
struct CreateThreadResponse {
	id: String,
}

#[derive(Deserialize)]
struct CreateThreadRunResponse {
	id: String,
}

#[derive(Deserialize)]
struct GetThreadRunResponse {
	status: String,
}

pub fn verify_env_vars() {
	var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
	var("OPENAI_ASSISTANT_ID").expect("OPENAI_ASSISTANT_ID must be set");
}

pub async fn create_thread(client: &Client) -> Option<String> {
	let result = client
		.post(format!("{OPENAI_API}/threads"))
		.header("Authorization", get_auth_header())
		.header("OpenAI-Beta", "assistants=v2")
		.send()
		.await;

	match result {
		Ok(response) => match response.json::<CreateThreadResponse>().await {
			Ok(json) => Some(json.id),
			Err(e) => {
				println!("Error creating thread: {e:#?}");
				None
			}
		},
		Err(e) => {
			println!("Error creating thread: {e:#?}");
			None
		}
	}
}

pub async fn add_message_to_thread(message: &str, thread_id: &str, client: &Client) {
	let result = client
		.post(format!("{OPENAI_API}/threads/{thread_id}/messages"))
		.header("Authorization", get_auth_header())
		.header("OpenAI-Beta", "assistants=v2")
		.json(&json!({
			"content": message,
			"role": "user",
		}))
		.send()
		.await;

	if let Err(e) = result {
		println!("Error adding message to thread: {e:#?}");
	};
}

pub async fn create_run(instructions: &str, thread_id: &str, client: &Client) -> Option<String> {
	let result = client
		.post(format!("{OPENAI_API}/threads/{thread_id}/runs"))
		.header("Authorization", get_auth_header())
		.header("OpenAI-Beta", "assistants=v2")
		.json(&json!({
			"assistant_id": var("OPENAI_ASSISTANT_ID").unwrap(),
			"additional_instructions": instructions,
		}))
		.send()
		.await;

	match result {
		Ok(response) => match response.json::<CreateThreadRunResponse>().await {
			Ok(json) => Some(json.id),
			Err(e) => {
				println!("Error creating run: {e:#?}");
				None
			}
		},
		Err(e) => {
			println!("Error creating run: {e:#?}");
			None
		}
	}
}

pub async fn check_run_status(run_id: &str, thread_id: &str, client: &Client) -> String {
	let result = client
		.get(format!("{OPENAI_API}/threads/{thread_id}/runs/{run_id}"))
		.header("Authorization", get_auth_header())
		.header("OpenAI-Beta", "assistants=v2")
		.send()
		.await;

	match result {
		Ok(response) => match response.json::<GetThreadRunResponse>().await {
			Ok(thread_run) => thread_run.status,
			Err(e) => {
				println!("Error checking run status: {e:#?}");
				"failed".to_string()
			}
		},
		Err(e) => {
			println!("Error checking run status: {e:#?}");
			"failed".to_string()
		}
	}
}

pub async fn get_thread_run_result(
	run_id: &str,
	thread_id: &str,
	client: &Client,
) -> Option<String> {
	let response = client
		.get(format!(
			"{OPENAI_API}/threads/{thread_id}/messages?run_id={run_id}"
		))
		.header("Authorization", get_auth_header())
		.header("OpenAI-Beta", "assistants=v2")
		.send()
		.await;

	match response {
		Ok(response) => match response.json::<GetThreadMessagesResponse>().await {
			Ok(body) => {
				if body.data.len() > 0 && body.data[0].content.len() > 0 {
					Some(body.data[0].content[0].text.value.clone())
				} else {
					None
				}
			}
			Err(e) => {
				println!("Error getting thread run result: {e:#?}");
				None
			}
		},
		Err(e) => {
			println!("Error getting thread run result: {e:#?}");
			None
		}
	}
}

fn get_auth_header() -> String {
	format!("Bearer {}", var("OPENAI_API_KEY").unwrap())
}
