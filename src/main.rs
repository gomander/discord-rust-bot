use postgrest::Postgrest;

use serenity::all::User;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::ChannelId;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

mod database;
mod openai;

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
    let is_dm = msg.guild_id == None;

    !is_bot && (contains_my_name || mentions_me || replies_to_me || is_dm)
}

async fn create_thread(channel_id: &str, database: &Postgrest) -> String {
    let thread_id = openai::create_thread(&reqwest::Client::new()).await;
    database::set_thread(&thread_id, channel_id, database).await;
    thread_id
}

async fn get_response(msg: &Message, thread_id: &str) -> String {
    let client = reqwest::Client::new();

    openai::add_message_to_thread(msg, thread_id, &client).await;

    let run_id = openai::create_run(&msg.author, thread_id, &client).await;

    let terminal_statuses = ["completed", "expired", "failed", "cancelled", "incomplete"];

    for i in 0..10 {
        let status = openai::check_run_status(&run_id, thread_id, &client).await;
        if terminal_statuses.contains(&status.as_str()) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(i + 3));
    }

    openai::get_thread_run_result(&run_id, thread_id, &client).await
}

async fn send_response(response: &str, channel_id: &ChannelId, context: &Context) {
    for chunk in response.chars().collect::<Vec<_>>().chunks(1000) {
        let message = MessageBuilder::new()
            .push(chunk.iter().collect::<String>())
            .build();

        if let Err(e) = channel_id.say(&context.http, &message).await {
            println!("Error sending message: {e:?}");
        }
    }
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, context: Context, msg: Message) {
        println!("Message received: {:?}", msg.content);
        if should_reply(&msg, &context.cache.current_user()) {
            println!("Should reply");

            let database = database::initialize_database();
            let thread_id =
                database::get_thread_id_for_channel(&msg.channel_id.to_string(), &database).await;

            let thread_id = match thread_id {
                Some(id) => {
                    println!("Thread exists in DB");
                    id
                }
                None => {
                    println!("Thread does not exist in DB");
                    create_thread(&msg.channel_id.to_string(), &database).await
                }
            };

            let response = get_response(&msg, &thread_id).await;
            send_response(&response, &msg.channel_id, &context).await;
        } else {
            println!("Should not reply");
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set!");
    openai::verify_env_vars();
    database::verify_env_vars();

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(e) = client.start().await {
        println!("Client error: {e:?}");
    }
}
