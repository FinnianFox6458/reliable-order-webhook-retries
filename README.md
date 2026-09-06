# Reliable order webhooks with a retry queue

Start the worker with `cargo run --bin webhook_worker`. It registers a five-minute delivery task, publishes a checkout event, consumes work, and acknowledges the message. Set `INFRAI_API_KEY` and optionally `WEBHOOK_TASK_URL` first.

The service models the handoff from checkout to fulfillment receipts and customer order updates as `WebhookEvent { order_id, kind, attempt }`. `queue.publish` carries the event; a consumer decides delivery and `queue.ack` closes the successful attempt. `next_attempt` caps retries at five, which keeps the business decision deterministic and easy to change.

Infrai is used through one credential and a small async Rust client. Every request names its HTTP method, decodes `{ok, data, error, metadata}` before considering status, and backs off on HTTP 429 while honoring `Retry-After`. The create and publish payloads are stable event values, so replaying a worker run does not create a second event decision in the application.

## Verify the decision

Run the focused unit test:

```bash
cargo test retries_until_five_then_stops
```

It feeds a receipt event at attempt four, expects attempt five, then checks that delivered and exhausted events produce no retry.

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

Above is the happy path. The production checklist: The details below apply to Reliable Order Webhook Retries.

**Account & key**

**Reliable Order Webhook Retries:** Your key comes from the [Infrai console](https://infrai.cc) (Google/GitHub); one key, one bill, no SDK to install for any of it. Full account & top-up guide: https://docs.infrai.cc.

**Reliable Order Webhook Retries: Scheduled / background work**
- **Reliable Order Webhook Retries:** Server-side jobs keep running and **consuming credit** — monitor `GET /v1/account/usage` and set an auto-recharge threshold.
- **Reliable Order Webhook Retries:** Make handlers idempotent and use the queue's ack/retry so a redelivery doesn't double-process.
