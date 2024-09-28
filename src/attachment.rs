use serenity::model::channel::Attachment;

pub async fn get_attachment_data(attachment: &Attachment) -> Option<String> {
    println!("Attachment: {:#?}", attachment);
    let response = attachment.download().await;

    match response {
        Ok(data) => {
            let decoded = String::from_utf8(data);

            match decoded {
                Ok(decoded) => Some(decoded),
                Err(e) => {
                    println!("Error decoding attachment: {e:#?}");
                    None
                }
            }
        }
        Err(e) => {
            println!("Error downloading attachment: {e:#?}");
            None
        }
    }
}
