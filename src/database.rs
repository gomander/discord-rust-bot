use std::env::var;

use postgrest::Postgrest;

pub fn verify_env_vars() {
    var("DATABASE_URL").expect("DATABASE_URL must be set");
    var("DATABASE_PUBLIC_KEY").expect("DATABASE_PUBLIC_KEY must be set");
    var("DATABASE_SERVICE_KEY").expect("DATABASE_SERVICE_KEY must be set");
    println!("Database environment variables are set");
}

pub fn initialize_database() -> Postgrest {
    let database_url = var("DATABASE_URL").unwrap();
    let public_key = var("DATABASE_PUBLIC_KEY").unwrap();
    let service_key = var("DATABASE_SERVICE_KEY").unwrap();

    let database = Postgrest::new(database_url)
        .insert_header("apikey", public_key)
        .insert_header("Authorization", service_key);
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
            println!("Error getting thread ID for channel: {e:?}");
            return None;
        }
    };

    match body {
        Ok(data) => {
            println!(r#"data: "{data}""#);
            if data.len() < 3 {
                return None;
            }
            if let Some(thread_id) = serde_json::json!(&data)[0]["thread_id"].as_str() {
                Some(thread_id.to_string())
            } else {
                None
            }
        }
        Err(e) => {
            println!("Error getting thread ID for channel: {e:?}");
            None
        }
    }
}

pub async fn set_thread(thread_id: &str, channel_id: &str, client: &Postgrest) {
    let json = format!(r#"{{ "thread_id": "{thread_id}", "channel_id": "{channel_id}" }}"#);
    println!("Setting thread for channel: {json}");
    let result = client
        .from("threads_to_channels")
        .insert(json)
        .execute()
        .await;

    match result {
        Ok(response) => println!("{:?}", response.text().await.unwrap()),
        Err(e) => println!("Error setting thread for channel: {e:?}"),
    };
}
