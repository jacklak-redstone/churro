A very early version of a Rust crate to make [Chatto](https://chatto.run) bots.

# Examples

```rust
use churro::{Bot, CResult, Ctx, EventHandler, async_trait};

struct Hdlr;

#[async_trait]
impl EventHandler for Hdlr {
    async fn message_sent(&self, ctx: Ctx) {
        let bot = &ctx.bot;
        let Some(user) = &ctx.user else { return };

        if bot.user_id() == user.id {
            return;
        }

        ctx.reply("Nice message!");
    }
}

#[tokio::main]
async fn main() -> CResult {
    let bot = Bot::login("bob", "bobs_strong_password", "chat.example.com");
    bot.start_listening(Hdlr);
    tokio::signal::ctrl_c().await.unwrap();
    Ok(())
}
```

# Todo

* Have handlers for all events
* Caching!
* More functions!
* Document everything we can
* Move away from connectrpc-build?

Contributions are welcome!