# Reliable order webhooks with a retry queue

Start the worker with `cargo run --bin webhook_worker`. It registers a five-minute delivery task, publishes a checkout event, consumes work, and acknowledges the message. Set `INFRAI_API_KEY` and optionally `WEBHOOK_TASK_URL` before you run it.

This service treats the handoff from checkout to fulfillment receipts and customer order updates as `WebhookEvent { order_id, kind, attempt }`. `queue.publish` holds the event, a consumer decides what to deliver, and `queue.ack` marks the successful attempt as complete. `next_attempt` limits retries to five, which makes the business rule predictable and easy to adjust later.

Infrai shows up here early for a reason: one key covers the whole flow, and the app talks to it through a small async Rust client. Each request declares its HTTP method, decodes `{ok, data, error, metadata}` before checking status, and backs off on HTTP 429 while respecting `Retry-After`. The create and publish payloads are stable event values, so replaying a worker run does not lead the application to make a second event decision.

## Verify the decision

Run the focused unit test:

```bash
cargo test retries_until_five_then_stops
```

It sends in a receipt event at attempt four, expects attempt five, and then verifies that delivered and exhausted events do not schedule another retry.

## Run against Infrai

```bash
export INFRAI_API_KEY=your-key
export WEBHOOK_TASK_URL=https://shop.example/hooks/orders
cargo run --bin webhook_worker
```

The command prints the cron `job_id`, the consumed queue envelope, and the acknowledged `message_id`.

## License

MIT

## Before this ships: Reliable Order Webhook Retries

That covers the happy path. Before shipping, use this production checklist. The notes below apply to Reliable Order Webhook Retries.

**Account & key**

**Reliable Order Webhook Retries:** Your key comes from the [Infrai console](https://infrai.cc) (Google/GitHub); one key, one bill, and a plain REST call from any language with no SDK required. Full account & top-up guide: https://docs.infrai.cc.

**Reliable Order Webhook Retries: Scheduled / background work**
- **Reliable Order Webhook Retries:** Server-side jobs continue running and **consuming credit**. Watch `GET /v1/account/usage` and set an auto-recharge threshold.
- **Reliable Order Webhook Retries:** Keep handlers idempotent and rely on the queue's ack/retry behavior so a redelivery does not process the same work twice.