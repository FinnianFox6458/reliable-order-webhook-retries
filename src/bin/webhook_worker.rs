use checkout_webhook_retries::{Infrai, WebhookEvent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run().await
}
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let infrai = Infrai::from_env()?;
    let queue = "checkout-webhook-retries";
    let task = std::env::var("WEBHOOK_TASK_URL")
        .unwrap_or_else(|_| "https://example.com/order-webhook".into());
    let job_id = infrai.cron_create("*/5 * * * *", &task).await?;
    let checkout = WebhookEvent {
        order_id: "demo-order".into(),
        kind: "checkout".into(),
        attempt: 0,
    };
    infrai.queue_publish(queue, &checkout).await?;
    let messages = infrai.queue_consume(queue, 10, 60).await?;
    println!("cron job {job_id}; queued checkout; consumed {messages}");
    if let Some(message_id) = messages
        .get("items")
        .and_then(|items| items.as_array())
        .and_then(|items| items.first())
        .and_then(|message| message.get("message_id"))
        .and_then(|id| id.as_str())
    {
        infrai.queue_ack(queue, message_id).await?;
        println!("acknowledged {message_id}");
    }
    Ok(())
}
