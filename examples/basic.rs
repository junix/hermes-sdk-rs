use hermes_sdk::{ContentPart, CreateResponseRequest, HermesClient, OutputItem};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = HermesClient::new("hello-key", "http://localhost:8642");

    // Health check
    println!("=== Health Check ===");
    let healthy = client.health().await?;
    println!("Gateway healthy: {healthy}\n");

    // 1. Simple request
    println!("=== Simple Request ===");
    let req = CreateResponseRequest::builder()
        .input("list files in current directory using one short command")
        .store(true)
        .build()?;

    let resp = client.create_response(&req).await?;
    println!("Response ID: {}", resp.id);
    println!("Status: {}", resp.status);
    println!("Usage: {} in / {} out / {} total tokens", resp.usage.input_tokens, resp.usage.output_tokens, resp.usage.total_tokens);

    println!("\n--- Output items ---");
    for item in &resp.output {
        match item {
            OutputItem::FunctionCall { name, arguments, call_id } => {
                println!("[tool call] {name}({arguments}) [{call_id}]");
            }
            OutputItem::FunctionCallOutput { call_id, output } => {
                let preview = if output.len() > 200 { &output[..200] } else { output };
                println!("[tool result] [{call_id}] {preview}");
            }
            OutputItem::Message { role, content } => {
                for part in content {
                    let text = match part {
                                ContentPart::OutputText { text } => text.as_str(),
                            };
                        println!("[{role}] {text}");
                }
            }
        }
    }

    // 2. Multi-turn via conversation name
    let conv_name = "sdk-demo";
    println!("\n=== Multi-turn with conversation '{conv_name}' ===");

    let req2 = CreateResponseRequest::builder()
        .input("what did you just do?")
        .conversation(conv_name)
        .build()?;

    let resp2 = client.create_response(&req2).await?;
    println!("Response ID: {}", resp2.id);
    if let Some(text) = resp2.text() {
        println!("Assistant: {text}");
    }

    // 3. Continue the same conversation
    let req3 = CreateResponseRequest::builder()
        .input("now run: uname -a")
        .conversation(conv_name)
        .build()?;

    let resp3 = client.create_response(&req3).await?;
    if let Some(text) = resp3.text() {
        println!("\nAssistant: {text}");
    }

    // 4. Retrieve a stored response
    println!("\n=== Retrieve stored response ===");
    let stored = client.get_response(&resp.id).await?;
    println!("Retrieved: {} (status: {})", stored.id, stored.status);

    // 5. Delete
    println!("\n=== Delete response ===");
    let deleted = client.delete_response(&resp.id).await?;
    println!("Deleted: {} (ok: {})", deleted.id, deleted.deleted);

    println!("\nDone!");
    Ok(())
}
