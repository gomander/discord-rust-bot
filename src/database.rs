use postgrest::Postgrest;
use serde::Deserialize;
use serde_json::from_str;
use std::env::var;

#[derive(Deserialize)]
struct ThreadToChannel {
	thread_id: String,
	channel_id: String,
}

impl Into<String> for ThreadToChannel {
	fn into(self) -> String {
		format!(
			r#"{{ "thread_id": "{}", "channel_id": "{}" }}"#,
			self.thread_id, self.channel_id
		)
	}
}

pub fn verify_env_vars() {
	var("DATABASE_URL").expect("DATABASE_URL must be set");
	var("DATABASE_SERVICE_KEY").expect("DATABASE_SERVICE_KEY must be set");
}

pub fn initialize_database() -> Postgrest {
	let database_url = var("DATABASE_URL").unwrap();
	let service_key = var("DATABASE_SERVICE_KEY").unwrap();

	let database = Postgrest::new(database_url)
		.insert_header("apikey", &service_key)
		.insert_header("Authorization", format!("Bearer {service_key}"));
	println!("Database initialized");
	database
}

pub async fn get_thread_id_for_channel(channel_id: &str, client: &Postgrest) -> Option<String> {
	let result = client
		.from("threads_to_channels")
		.select("*")
		.eq("channel_id", channel_id)
		.execute()
		.await;

	let body = match result {
		Ok(response) => response.text().await,
		Err(e) => {
			println!("Error getting thread ID for channel: {e:#?}");
			return None;
		}
	};

	match body {
		Ok(data) => match from_str::<Vec<ThreadToChannel>>(&data) {
			Ok(rows) => {
				if rows.len() > 0 {
					Some(rows[0].thread_id.clone())
				} else {
					None
				}
			}
			Err(e) => {
				println!("Error parsing thread ID for channel: {e:#?}");
				None
			}
		},
		Err(e) => {
			println!("Error getting thread ID for channel: {e:#?}");
			None
		}
	}
}

pub async fn set_thread(thread_id: &str, channel_id: &str, client: &Postgrest) {
	let result = client
		.from("threads_to_channels")
		.insert(ThreadToChannel {
			thread_id: thread_id.to_string(),
			channel_id: channel_id.to_string(),
		})
		.execute()
		.await;

	if let Err(e) = result {
		println!("Error setting thread for channel: {e:#?}");
	};
}
