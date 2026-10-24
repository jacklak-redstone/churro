A very early version of an unofficial Rust crate to make [Chatto](https://chatto.run) bots.

# Examples

```rust
use churro::prelude::*;

struct Hdlr;

#[async_trait]
impl EventHandler for Hdlr {
    async fn message_sent(&self, ctx: Ctx) {
        let Some(user) = &ctx.user else {
            return;
        };
        if user.is_bot() {
            return;
        };
        let _ = ctx.reply("Hello!", false).await;
    }
}

#[tokio::main]
async fn main() -> CResult {
    let bot = Bot::login(
        "cht_BK_token123",
        "https://chat.example.com",
    )
    .await?;

    bot.listen(Hdlr).await?;

    tokio::signal::ctrl_c().await.unwrap();

    Ok(())
}
```

# Todo

* More robust everything
* Have handlers for all events
* Caching!
* More functions!
* Document everything we can
* Move away from connectrpc-build?

Contributions are welcome!
