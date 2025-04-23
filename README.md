# Webhook Configurator
This is a test utility for Discord's WebHook.
I made it for testing large-scale WebHook application.

## How to

```bash
# Create many channels
. .env && cargo run explosion <SERVER_ID>
# Create many webhooks
. .env && cargo run webhook <SERVER_ID> | tee webhooks.txt
```

## Template
https://discord.new/Jh3tUyFPe9em

