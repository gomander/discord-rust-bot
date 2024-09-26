use std::env;

use serenity::all::User;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::ChannelId;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

mod openai_api;
use openai_api::*;

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

	!is_bot && (contains_my_name || mentions_me || replies_to_me)
}

async fn get_response(msg: &Message) -> String {
    let client = reqwest::Client::new();

    add_message_to_thread(msg, &client).await;

    let run_id = create_run(&msg.author, &client).await;

    let terminal_statuses = ["completed", "expired", "failed", "cancelled", "incomplete"];

    for i in 0..10 {
        let status = check_run_status(&run_id, &client).await;
        if terminal_statuses.contains(&status.as_str()) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(i + 3));
    }

    get_thread_run_result(&run_id, &client).await
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
        if should_reply(&msg, &context.cache.current_user()) {
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
    // let openai_thread_id = create_thread(&reqwest::Client::new()).await;
    // env::set_var("OPENAI_THREAD_ID", openai_thread_id);
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
