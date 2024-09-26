use postgrest::Postgrest;

pub fn verify_env_vars() {
    std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    std::env::var("DATABASE_PUBLIC_KEY").expect("DATABASE_PUBLIC_KEY must be set");
    std::env::var("DATABASE_SERVICE_KEY").expect("DATABASE_SERVICE_KEY must be set");
    println!("Database environment variables are set");
}

pub fn initialize_database() -> Postgrest {
    let database_url = std::env::var("DATABASE_URL").unwrap();
    let database = Postgrest::new(database_url)
        .insert_header("apikey", std::env::var("DATABASE_PUBLIC_KEY").unwrap())
        .insert_header(
            "Authorization",
            format!("Bearer {}", std::env::var("DATABASE_SERVICE_KEY").unwrap()),
        );
    println!("Database initialized");
    database
}

pub async fn get_thread_id_for_channel(channel_id: &str, client: &Postgrest) -> Option<String> {
    let result = client
        .from("threads_to_channels")
        .select("*")
        .eq("channel_id", channel_id)
        .single()
        .execute()
        .await;

    let body = match result {
        Ok(response) => response.text().await,
        Err(_) => {
            println!("Error getting thread ID for channel");
            return None;
        }
    };

    match body {
        Ok(data) => {
            print!("data: {}", data);
            Some(data)
        }
        Err(_) => None,
    }
}

pub async fn set_thread(thread_id: &str, channel_id: &str, client: &Postgrest) {
    let result = client
        .from("threads_to_channels")
        .insert(format!(
            r#"["thread_id": "{}", "channel_id": "{}"]"#,
            thread_id, channel_id
        ))
        .execute()
        .await;

    match result {
        Ok(_) => println!("Thread set for channel"),
        Err(e) => println!("Error setting thread for channel: {:?}", e),
    }
}
