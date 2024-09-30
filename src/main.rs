use postgrest::Postgrest;
use reqwest::Client as ReqwestClient;
use serenity::all::{async_trait, Client, Context, EventHandler, Message, Ready, User};
use std::{collections::HashMap, env::var, sync::Arc};
use tokio::sync::Mutex;

mod database;
mod discord;
mod openai;

#[tokio::main]
async fn main() {
	verify_env_vars();

	let mut client = Client::builder(discord::get_token(), discord::get_intents())
		.event_handler(Handler {
			database: Arc::new(database::initialize_database()),
			cache: Arc::new(Mutex::new(HashMap::new())),
			reqwest: ReqwestClient::new(),
			debug: is_debug_mode(),
		})
		.await
		.expect("Error creating client");

	if let Err(e) = client.start().await {
		println!("Client error: {e:#?}");
	};
}

struct Handler {
	database: Arc<Postgrest>,
	cache: Arc<Mutex<HashMap<String, String>>>,
	reqwest: ReqwestClient,
	debug: bool,
}

#[async_trait]
impl EventHandler for Handler {
	async fn message(&self, context: Context, msg: Message) {
		let current_user = context.cache.current_user().clone();

		if msg.author.id == current_user.id {
			return;
		};

		if self.debug {
			println!(
				"Message received from {}",
				discord::get_user_name(&msg.author)
			);
		};

		if let Some(thread_id) = get_thread_id(
			&msg.channel_id.to_string(),
			&self.cache,
			&self.database,
			&self.reqwest,
			self.debug,
		)
		.await
		{
		if should_store(&msg, &current_user) {
			if self.debug {
				println!("Adding message to thread");
			};

			openai::add_message_to_thread(
				&create_thread_message(&msg, self.debug).await,
				&thread_id,
				&self.reqwest,
			)
			.await;
		};

		if should_reply(&msg, &current_user) {
			if self.debug {
				println!("Generating response");
			};

			msg
				.channel_id
				.broadcast_typing(&context.http)
				.await
					.unwrap_or_default();

				if let Some(response) = get_response(&msg, &thread_id, &self.reqwest, self.debug).await {
			discord::send_message(&response, &msg.channel_id, &context).await;
				};
			};
		} else {
			println!("Error getting thread ID");
		};
	}

	async fn ready(&self, _: Context, ready: Ready) {
		println!("{} is connected!", ready.user.name);
	}
}

async fn get_thread_id(
	channel_id: &str,
	cache: &Arc<Mutex<HashMap<String, String>>>,
	database: &Postgrest,
	reqwest: &ReqwestClient,
	debug: bool,
) -> Option<String> {
	let cache_guard = cache.lock().await;

	if let Some(thread_id) = cache_guard.get(channel_id) {
		if debug {
			println!("Using cached thread ID: {thread_id}");
		};
		Some(thread_id.clone())
	} else {
		drop(cache_guard);
		if let Some(thread_id) = match database::get_thread_id_for_channel(channel_id, database).await {
			Some(id) => {
				if debug {
					println!("Using thread ID from database: {id}");
				};
				Some(id)
			}
			None => {
				if debug {
					println!("Creating new thread ID");
				};
				create_thread(channel_id, database, &reqwest).await
			}
		} {
		let mut cache_guard = cache.lock().await;
		cache_guard.insert(channel_id.to_string(), thread_id.clone());
			Some(thread_id)
		} else {
			None
		}
	}
}

fn should_store(msg: &Message, current_user: &User) -> bool {
	msg.author.id != current_user.id && msg.content.len() > 0 && !msg.author.bot
}

async fn create_thread_message(msg: &Message, debug: bool) -> String {
	format!(
		"{} ({}): \"\"\"\n{}\n\"\"\"{}",
		discord::get_user_name(&msg.author),
		&msg.author.id,
		&msg.content,
		format_attachments(&msg, debug).await
	)
}

async fn format_attachments(msg: &Message, debug: bool) -> String {
	let mut formatted_attachments = String::new();
	for file in &msg.attachments {
		let attachment_data = discord::get_attachment_data(&file, debug).await;
		match attachment_data {
			Some(data) => {
				formatted_attachments = format!(
					"{formatted_attachments}\nAttachment \"{}\": \"\"\"\n{data}\n\"\"\"",
					file.filename
				);
			}
			None => {
				println!("Error getting attachment data");
			}
		};
	}
	formatted_attachments
}

fn should_reply(msg: &Message, current_user: &User) -> bool {
	let is_bot = msg.author.bot;
	let content = msg.content.to_ascii_lowercase();
	let own_name = current_user.name.to_ascii_lowercase();
	let contains_my_name = content.contains(&own_name);
	let own_id = current_user.id;
	let mentions_me = msg.mentions_user(current_user);
	let replies_to_me = msg
		.referenced_message
		.clone()
		.is_some_and(|m| m.author.id == own_id);
	let is_dm = msg.guild_id.is_none();

	!is_bot && (contains_my_name || mentions_me || replies_to_me || is_dm)
}

async fn create_thread(
	channel_id: &str,
	database: &Postgrest,
	reqwest: &ReqwestClient,
) -> Option<String> {
	if let Some(thread_id) = openai::create_thread(&reqwest).await {
	database::set_thread(&thread_id, channel_id, database).await;
		Some(thread_id)
	} else {
		None
	}
}

const TERMINAL_STATUSES: [&str; 5] = ["completed", "expired", "failed", "cancelled", "incomplete"];

async fn get_response(
	msg: &Message,
	thread_id: &str,
	reqwest: &ReqwestClient,
	debug: bool,
) -> Option<String> {
	if let Some(run_id) = openai::create_run(
		&format!(
			"The most recent message was sent by {} (ID: {})",
			discord::get_user_name(&msg.author),
			msg.author.id
		),
		thread_id,
		&reqwest,
	)
	.await
	{
		for _ in 0..10 {
			std::thread::sleep(std::time::Duration::from_secs(4));
		let status = openai::check_run_status(&run_id, thread_id, &reqwest).await;
			if TERMINAL_STATUSES.contains(&status.as_str()) {
				if &status == "completed" {
					return openai::get_thread_run_result(&run_id, thread_id, &reqwest).await;
				} else {
					if debug {
						println!("Run failed with status: {status}");
					};
					return None;
				};
			}
	}

	openai::get_thread_run_result(&run_id, thread_id, &reqwest).await
	} else {
		None
	}
}

fn verify_env_vars() {
	dotenv::dotenv().ok();
	openai::verify_env_vars();
	database::verify_env_vars();
}

fn is_debug_mode() -> bool {
	let debug = var("DEBUG").unwrap_or_else(|_| "false".to_string()) == "true";
	if debug {
		println!("Running in debug mode");
	};
	debug
}
