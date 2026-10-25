use crate::prelude::*;
use crate::proto::chatto::realtime::v1::realtime_event::Event::*;

use std::sync::Arc;

/// Struct for dedicated bots.
#[repr(transparent)]
#[derive(Clone)]
pub struct Bot {
    pub client: Client,
}

impl Bot {
    /// Log in to a server with a bot token.
    pub async fn login(token: &str, server: &str) -> CResult<Self> {
        Ok(Bot {
            client: crate::Client::with_token(token, server).await?,
        })
    }

    /// Start listening to events with a given EventHandler.
    pub async fn listen(&self, hdlr: impl EventHandler) -> CResult {
        let bot = Arc::new(self.clone());
        let handler = Arc::new(hdlr);
        let client = self.client.clone();

        tokio::spawn(async move {
            let mut stream = match client.event_stream().await {
                Ok(stream) => stream,
                Err(err) => {
                    eprintln!("Failed to create event stream: {err}");
                    return;
                }
            };
            while let Some(e) = stream.next().await {
                let Some(ev) = e.event else {
                    continue;
                };
                let bot_for_hdlr = Arc::clone(&bot);
                let hdlr = Arc::clone(&handler);
                let mut ctx = Ctx::with_bot(Arc::clone(&bot));

                tokio::spawn(async move {
                    ctx.user = bot_for_hdlr
                        .client
                        .fetch_user(&e.actor_id.unwrap_or_default())
                        .await
                        .ok();

                    match ev {
                        MessagePosted(ev) => {
                            let (room, msg, root_msg) = tokio::join!(
                                async { bot_for_hdlr.client.fetch_room(&ev.room_id).await.ok() },
                                async {
                                    bot_for_hdlr.client.fetch_msg(&ev.room_id, &e.id).await.ok()
                                },
                                async {
                                    bot_for_hdlr
                                        .client
                                        .fetch_msg(&ev.room_id, &ev.thread_root_event_id)
                                        .await
                                        .ok()
                                },
                            );

                            ctx.room = room;
                            ctx.message = msg;
                            ctx.root_message = root_msg;

                            hdlr.message_sent(ctx).await;
                        }
                        MessageEdited(ev) => {
                            let (room, msg) = tokio::join!(
                                async { bot_for_hdlr.client.fetch_room(&ev.room_id).await.ok() },
                                async {
                                    bot_for_hdlr
                                        .client
                                        .fetch_msg(&ev.room_id, &ev.message_event_id)
                                        .await
                                        .ok()
                                }
                            );

                            ctx.room = room;
                            ctx.message = msg;

                            hdlr.message_edited(ctx).await;
                        }
                        MessageRetracted(ev) => {
                            ctx.room = bot_for_hdlr.client.fetch_room(&ev.room_id).await.ok();

                            hdlr.message_deleted(ctx, &ev.message_event_id).await;
                        }
                        ReactionAdded(ev) => {
                            let (room, msg) = tokio::join!(
                                async { bot_for_hdlr.client.fetch_room(&ev.room_id).await.ok() },
                                async {
                                    bot_for_hdlr
                                        .client
                                        .fetch_msg(&ev.room_id, &ev.message_event_id)
                                        .await
                                        .ok()
                                }
                            );

                            ctx.room = room;
                            ctx.message = msg;

                            hdlr.reaction_added(ctx, &ev.emoji).await;
                        }
                        ReactionRemoved(ev) => {
                            let (room, msg) = tokio::join!(
                                async { bot_for_hdlr.client.fetch_room(&ev.room_id).await.ok() },
                                async {
                                    bot_for_hdlr
                                        .client
                                        .fetch_msg(&ev.room_id, &ev.message_event_id)
                                        .await
                                        .ok()
                                }
                            );

                            ctx.room = room;
                            ctx.message = msg;

                            hdlr.reaction_removed(ctx, &ev.emoji).await;
                        }
                        MessagePinned(ev) => {
                            let (room, msg) = tokio::join!(
                                async { bot_for_hdlr.client.fetch_room(&ev.room_id).await.ok() },
                                async {
                                    bot_for_hdlr
                                        .client
                                        .fetch_msg(&ev.room_id, &ev.message_event_id)
                                        .await
                                        .ok()
                                }
                            );

                            ctx.room = room;
                            ctx.message = msg;

                            hdlr.message_pinned(ctx).await;
                        }
                        MessageUnpinned(ev) => {
                            let (room, msg) = tokio::join!(
                                async { bot_for_hdlr.client.fetch_room(&ev.room_id).await.ok() },
                                async {
                                    bot_for_hdlr
                                        .client
                                        .fetch_msg(&ev.room_id, &ev.message_event_id)
                                        .await
                                        .ok()
                                }
                            );

                            ctx.room = room;
                            ctx.message = msg;

                            hdlr.message_unpinned(ctx).await;
                        }
                        _ => {}
                    }
                });
            }
        });
        Ok(())
    }
}
