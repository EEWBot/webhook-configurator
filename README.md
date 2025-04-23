# Webhook Configurator
Test tool for developing large-scale Webhook application.

## How to

```bash
# Create many channels
. .env && cargo run explosion <SERVER_ID>
# Create many webhooks
. .env && cargo run webhook <SERVER_ID> | tee webhooks.txt
# Export many webhooks
. .env && cargo run export <SERVER_ID> | tee webhooks.txt
```

## Created Template
https://discord.new/Jh3tUyFPe9em

