use postgrest::Postgrest;
use reqwest::Client as ReqwestClient;
use serenity::all::User;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::ChannelId;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;
use std::{collections::HashMap, env::var, sync::Arc};
use tokio::sync::Mutex;

mod attachment;
mod database;
mod openai;
mod util;

async fn get_thread_id(
    channel_id: &str,
    cache: &Arc<Mutex<HashMap<String, String>>>,
    database: &Postgrest,
    debug: bool,
) -> String {
    let cache_guard = cache.lock().await;

    if let Some(thread_id) = cache_guard.get(channel_id) {
        if debug {
            println!("Using cached thread ID: {thread_id}");
        };
        thread_id.clone()
    } else {
        drop(cache_guard);
        let thread_id = database::get_thread_id_for_channel(channel_id, database).await;
        let thread_id = match thread_id {
            Some(id) => {
                if debug {
                    println!("Using thread ID from database: {id}");
                };
                id
            }
            None => {
                if debug {
                    println!("Creating new thread ID");
                };
                create_thread(channel_id, database).await
            }
        };
        let mut cache_guard = cache.lock().await;
        cache_guard.insert(channel_id.to_string(), thread_id.clone());
        thread_id
    }
}

fn should_store(msg: &Message, current_user: &User) -> bool {
    msg.author.id != current_user.id && msg.content.len() > 0 && !msg.author.bot
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

async fn create_thread(channel_id: &str, database: &Postgrest) -> String {
    let thread_id = openai::create_thread(&ReqwestClient::new()).await;
    database::set_thread(&thread_id, channel_id, database).await;
    thread_id
}

async fn get_response(msg: &Message, thread_id: &str, reqwest: &ReqwestClient) -> String {
    let run_id = openai::create_run(
        &format!(
            "The most recent message was sent by {} (ID: {})",
            util::get_user_name(&msg.author),
            msg.author.id
        ),
        thread_id,
        &reqwest,
    )
    .await;

    let terminal_statuses = ["completed", "expired", "failed", "cancelled", "incomplete"];

    for i in 0..10 {
        let status = openai::check_run_status(&run_id, thread_id, &reqwest).await;
        if terminal_statuses.contains(&status.as_str()) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(i + 3));
    }

    openai::get_thread_run_result(&run_id, thread_id, &reqwest).await
}

async fn send_response(response: &str, channel_id: &ChannelId, context: &Context) {
    for chunk in response.chars().collect::<Vec<_>>().chunks(1000) {
        let message = MessageBuilder::new()
            .push(chunk.iter().collect::<String>())
            .build();

        if let Err(e) = channel_id.say(&context.http, &message).await {
            println!("Error sending message: {e:#?}");
        }
    }
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
        let thread_id = get_thread_id(
            &msg.channel_id.to_string(),
            &self.cache,
            &self.database,
            self.debug,
        )
        .await;

        let current_user = &context.cache.current_user().clone();

        if should_store(&msg, current_user) {
            if self.debug {
                println!("Adding message to thread");
            };
            let mut message = format!(
                "{} ({}): \"\"\"\n{}\n\"\"\"",
                util::get_user_name(&msg.author),
                &msg.author.id,
                &msg.content
            );
            for file in &msg.attachments {
                let attachment_data = attachment::get_attachment_data(&file).await;
                match attachment_data {
                    Some(data) => {
                        message = format!(
                            "{message}\nAttachment \"{}\": \"\"\"\n{data}\n\"\"\"",
                            file.filename
                        );
                    }
                    None => {
                        println!("Error getting attachment data");
                    }
                };
            }
            openai::add_message_to_thread(&message, &thread_id, &self.reqwest).await;
        }

        if should_reply(&msg, current_user) {
            if self.debug {
                println!("Generating response");
            };
            msg.channel_id
                .broadcast_typing(&context.http)
                .await
                .unwrap();

            let response = get_response(&msg, &thread_id, &self.reqwest).await;
            send_response(&response, &msg.channel_id, &context).await;
        };
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    verify_env_vars();
    let token = var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set!");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
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
