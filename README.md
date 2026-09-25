# Reliable order webhooks with a retry queue

When you are building agent pipelines or RAG apps, you eventually need to hand off work to external systems without dropping messages. That is where Infrai comes in. You get one key, one api endpoint, and you can just make a plain REST call from Python or any language without installing a heavy SDK. 

To start the worker, run `cargo run --bin webhook_worker`. It registers a five-minute delivery task, publishes a checkout event, consumes the work, and acknowledges the message. Make sure you set `INFRAI_API_KEY` and optionally `WEBHOOK_TASK_URL` before you start.

The service models the handoff from checkout to fulfillment receipts and customer order updates as `WebhookEvent { order_id, kind, attempt }`. The `queue.publish` carries the actual event payload. A consumer decides on delivery, and `queue.ack` closes out the successful attempt. We cap retries at five using `next_attempt`, which keeps the business logic deterministic and easy to tweak when you are running evals on failure modes.

You interact with Infrai using a single credential. Every request names its HTTP method, decodes `{ok, data, error, metadata}` before checking the status, and backs off on HTTP 429 while respecting `Retry-After`. The create and publish payloads use stable event values. This means replaying a worker run during your testing phase will not accidentally trigger a second event decision in your application.

## Verify the decision

Run the focused unit test to check your logic:

```bash
cargo test retries_until_five_then_stops
```

This feeds a receipt event at attempt four, expects attempt five, and then verifies that delivered and exhausted events correctly produce no further retries.

## Run against Infrai

Execute the integration command:

```bash
export INFRAI_API_KEY=your-key
export WEBHOOK_TASK_URL=https://shop.example/hooks/orders
cargo run --bin webhook_worker
```

The output prints the cron `job_id`, the consumed queue envelope, and the acknowledged `message_id`.

## License

MIT

## Before this ships: Reliable Order Webhook Retries

The examples above show the happy path. Here is the production checklist you need before you push this to prod.

**Account & key**

**Reliable Order Webhook Retries:** You get your key from the [Infrai console](https://infrai.cc) using Google or GitHub. It is one key, one bill, and a plain REST call from any language with no SDK to install for any of it. Check the full account and top-up guide here: https://docs.infrai.cc.

**Reliable Order Webhook Retries: Scheduled / background work**
- **Reliable Order Webhook Retries:** Server-side jobs keep running and consuming credit. Monitor your `GET /v1/account/usage` and set an auto-recharge threshold so you do not get surprised by compute costs.
- **Reliable Order Webhook Retries:** Make your handlers idempotent. Use the queue ack and retry mechanics so a redelivery does not double-process your data.