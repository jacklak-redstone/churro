use buffa::Message as _;
use connectrpc::Protocol;
use connectrpc::client::{ClientConfig, HttpClient};
use connectrpc::rustls::ClientConfig as TLSConfig;
use connectrpc::rustls::RootCertStore;
use futures_util::SinkExt;
use futures_util::stream::StreamExt;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use tokio_tungstenite::tungstenite::Message as WSMessage;

use crate::proto::chatto::admin::v1::*;
use crate::proto::chatto::api::v1::get_user_request::Target;
use crate::proto::chatto::api::v1::*;
use crate::proto::chatto::auth::v1::*;
use crate::proto::chatto::discovery::v1::*;
use crate::proto::chatto::realtime::v1::realtime_event_envelope::Event;
use crate::proto::chatto::realtime::v1::*;
use crate::{CResult, ChurroError, Ctx, EventHandler};

use std::sync::Arc;

#[derive(Clone)]
pub struct Bot {
    token: String,
    user_id: String,
    username: String,
    server_url: String,
    // Auth Service
    pub external_identity_auth_service: ExternalIdentityAuthServiceClient<HttpClient>,
    // Admin Services
    pub admin_diagnostics_service: AdminDiagnosticsServiceClient<HttpClient>,
    pub admin_event_log_service: AdminEventLogServiceClient<HttpClient>,
    pub admin_permission_service: AdminPermissionServiceClient<HttpClient>,
    pub admin_role_service: AdminRoleServiceClient<HttpClient>,
    pub admin_room_layout_service: AdminRoomLayoutServiceClient<HttpClient>,
    pub admin_server_service: AdminServerServiceClient<HttpClient>,
    pub admin_user_service: AdminUserServiceClient<HttpClient>,
    // API Services
    pub asset_service: AssetServiceClient<HttpClient>,
    pub asset_upload_service: AssetUploadServiceClient<HttpClient>,
    pub message_service: MessageServiceClient<HttpClient>,
    pub my_account_service: MyAccountServiceClient<HttpClient>,
    pub notification_preferences_service: NotificationPreferencesServiceClient<HttpClient>,
    pub notification_service: NotificationServiceClient<HttpClient>,
    pub push_notification_service: PushNotificationServiceClient<HttpClient>,
    pub role_service: RoleServiceClient<HttpClient>,
    pub room_directory_service: RoomDirectoryServiceClient<HttpClient>,
    pub room_service: RoomServiceClient<HttpClient>,
    pub server_service: ServerServiceClient<HttpClient>,
    pub thread_service: ThreadServiceClient<HttpClient>,
    pub user_service: UserServiceClient<HttpClient>,
    pub viewer_service: ViewerServiceClient<HttpClient>,
    pub voice_call_service: VoiceCallServiceClient<HttpClient>,
    // Discovery Service
    pub discovery_service: ServerDiscoveryServiceClient<HttpClient>,
}

#[derive(Deserialize)]
struct LoginResult {
    #[allow(unused)]
    success: bool,
    token: String,
    user: LoginUser,
}

#[derive(Deserialize)]
struct LoginUser {
    id: String,
    login: String,
}

#[inline(always)]
fn to_rt_frame<T: Into<realtime_client_frame::Frame>>(v: T) -> RealtimeClientFrame {
    RealtimeClientFrame {
        frame: Some(v.into()),
        ..Default::default()
    }
}

impl Bot {
    pub fn username(&self) -> String {
        self.username.clone()
    }

    pub fn user_id(&self) -> String {
        self.user_id.clone()
    }

    /// Login to the Chatto server.
    /// Currently only supports https and wss. Most servers should be using those anyway.
    /// # Example
    /// ```rust
    /// let bot = Bot::login("bob", "bobs_password", "chat.example.com").await?;
    /// ```
    pub async fn login(username: &str, password: &str, server: &str) -> CResult<Self> {
        let https_url = format!("https://{}", server);

        let resp = Client::new()
            .post(format!("{}/auth/login", https_url))
            .json(&json!({
                "login": username,
                "password": password,
            }))
            .send()
            .await?;

        let body = resp.text().await?;

        let result: LoginResult = serde_json::from_str(&body)?;

        let token = result.token;

        let mut roots = RootCertStore::empty();
        roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let tls_config = TLSConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();

        let conn = HttpClient::with_tls(tls_config.into());

        let config = ClientConfig::new(format!("{}/api/connect", https_url).parse()?)
            .with_protocol(Protocol::Connect)
            .with_default_header("Authorization", format!("Bearer {}", token.clone()));

        Ok(Bot {
            token,
            user_id: result.user.id,
            username: result.user.login,
            server_url: server.to_string(),
            // Auth
            external_identity_auth_service: ExternalIdentityAuthServiceClient::new(
                conn.clone(),
                config.clone(),
            ),
            // Admin
            admin_diagnostics_service: AdminDiagnosticsServiceClient::new(
                conn.clone(),
                config.clone(),
            ),
            admin_event_log_service: AdminEventLogServiceClient::new(conn.clone(), config.clone()),
            admin_permission_service: AdminPermissionServiceClient::new(
                conn.clone(),
                config.clone(),
            ),
            admin_role_service: AdminRoleServiceClient::new(conn.clone(), config.clone()),
            admin_room_layout_service: AdminRoomLayoutServiceClient::new(
                conn.clone(),
                config.clone(),
            ),
            admin_server_service: AdminServerServiceClient::new(conn.clone(), config.clone()),
            admin_user_service: AdminUserServiceClient::new(conn.clone(), config.clone()),
            // API
            asset_service: AssetServiceClient::new(conn.clone(), config.clone()),
            asset_upload_service: AssetUploadServiceClient::new(conn.clone(), config.clone()),
            message_service: MessageServiceClient::new(conn.clone(), config.clone()),
            my_account_service: MyAccountServiceClient::new(conn.clone(), config.clone()),
            notification_preferences_service: NotificationPreferencesServiceClient::new(
                conn.clone(),
                config.clone(),
            ),
            notification_service: NotificationServiceClient::new(conn.clone(), config.clone()),
            push_notification_service: PushNotificationServiceClient::new(
                conn.clone(),
                config.clone(),
            ),
            role_service: RoleServiceClient::new(conn.clone(), config.clone()),
            room_directory_service: RoomDirectoryServiceClient::new(conn.clone(), config.clone()),
            room_service: RoomServiceClient::new(conn.clone(), config.clone()),
            server_service: ServerServiceClient::new(conn.clone(), config.clone()),
            thread_service: ThreadServiceClient::new(conn.clone(), config.clone()),
            user_service: UserServiceClient::new(conn.clone(), config.clone()),
            viewer_service: ViewerServiceClient::new(conn.clone(), config.clone()),
            voice_call_service: VoiceCallServiceClient::new(conn.clone(), config.clone()),
            // Discovery
            discovery_service: ServerDiscoveryServiceClient::new(conn.clone(), config.clone()),
        })
    }

    /// Starts listening to events, using the given EventHandler.
    /// This will create a websocket connection, so expect some delay.
    pub async fn start_listening(&self, handler: impl EventHandler) -> CResult {
        let bot = Arc::new(self.clone());
        let handler = Arc::new(handler);

        let (mut sender, mut recvr) =
            tokio_tungstenite::connect_async(format!("wss://{}/api/realtime", self.server_url))
                .await?
                .0
                .split();
        sender
            .send(WSMessage::Binary(
                to_rt_frame(RealtimeClientHello {
                    protocol_version: 1,
                    bearer_token: Some(self.token.clone()),
                    ..Default::default()
                })
                .encode_to_bytes(),
            ))
            .await?;
        recvr.next().await.ok_or(ChurroError::WebsocketRecv(
            "couldn't receive server hello".to_string(),
        ))??;
        sender
            .send(WSMessage::Binary(
                to_rt_frame(RealtimeSubscribeEvents {
                    ..Default::default()
                })
                .encode_to_bytes(),
            ))
            .await?;
        recvr.next().await.ok_or(ChurroError::WebsocketRecv(
            "couldn't receive initial message".to_string(),
        ))??;
        tokio::spawn(async move {
            while let Some(Ok(m)) = recvr.next().await {
                let Ok(f) = RealtimeServerFrame::decode_from_slice(&m.into_data()) else {
                    continue;
                };
                let Some(f) = f.frame else { continue };
                let realtime_server_frame::Frame::Event(e) = f else {
                    continue;
                };
                let Some(event) = e.event else { continue };
                let actor_id = e.actor_id.clone();
                let hdlr = Arc::clone(&handler);
                let bot_for_hdlr = Arc::clone(&bot);

                let mut ctx = Ctx::with_bot(Arc::clone(&bot));

                tokio::spawn(async move {
                    let actor_id = actor_id.as_deref();

                    match &event {
                        Event::MentionNotification(ev) => {
                            let (room, user) = tokio::join!(
                                async { bot_for_hdlr.fetch_room(&ev.room_id).await.ok() },
                                async { bot_for_hdlr.fetch_user(&ev.actor_user_id).await.ok() },
                            );

                            ctx.room = room;
                            ctx.user = user;

                            hdlr.bot_mentioned(ctx).await;
                        }

                        Event::MessagePosted(ev) => {
                            let (room, message, root_message, user) = tokio::join!(
                                async { bot_for_hdlr.fetch_room(&ev.room_id).await.ok() },
                                async {
                                    bot_for_hdlr
                                        .fetch_message_raw(&ev.room_id, &ev.message_event_id)
                                        .await
                                        .ok()
                                },
                                async {
                                    let Some(mid) = &ev.thread_root_event_id else {
                                        return None;
                                    };
                                    bot_for_hdlr.fetch_message_raw(&ev.room_id, mid).await.ok()
                                },
                                async {
                                    if let Some(actor_id) = actor_id {
                                        bot_for_hdlr.fetch_user(actor_id).await.ok()
                                    } else {
                                        None
                                    }
                                }
                            );

                            let user = if user.is_none()
                                && let Some(m) = &message
                            {
                                bot_for_hdlr.fetch_user(&m.actor_id).await.ok()
                            } else {
                                None
                            };

                            ctx.room = room;
                            ctx.message = message;
                            ctx.root_message = root_message;
                            ctx.user = user;

                            hdlr.message_sent(ctx).await;
                        }

                        Event::MessageEdited(ev) => {
                            let (room, message, user) = tokio::join!(
                                async { bot_for_hdlr.fetch_room(&ev.room_id).await.ok() },
                                async {
                                    bot_for_hdlr
                                        .fetch_message_raw(&ev.room_id, &ev.message_event_id)
                                        .await
                                        .ok()
                                },
                                async {
                                    if let Some(actor_id) = actor_id {
                                        bot_for_hdlr.fetch_user(actor_id).await.ok()
                                    } else {
                                        None
                                    }
                                }
                            );

                            let user = if user.is_none()
                                && let Some(m) = &message
                            {
                                bot_for_hdlr.fetch_user(&m.actor_id).await.ok()
                            } else {
                                None
                            };

                            ctx.room = room;
                            ctx.message = message;
                            ctx.user = user;

                            hdlr.message_edited(ctx).await;
                        }

                        Event::MessageRetracted(ev) => {
                            let (room, user) = tokio::join!(
                                async { bot_for_hdlr.fetch_room(&ev.room_id).await.ok() },
                                async {
                                    if let Some(actor_id) = actor_id {
                                        bot_for_hdlr.fetch_user(actor_id).await.ok()
                                    } else {
                                        None
                                    }
                                }
                            );

                            ctx.room = room;
                            ctx.user = user;

                            hdlr.message_deleted(ctx, &ev.message_event_id.clone())
                                .await;
                        }

                        Event::NewDirectMessageNotification(ev) => {
                            let (room, user) = tokio::join!(
                                async { bot_for_hdlr.fetch_room(&ev.room_id).await.ok() },
                                async { bot_for_hdlr.fetch_user(&ev.sender_id).await.ok() }
                            );

                            ctx.room = room;
                            ctx.user = user;

                            hdlr.dm_started(ctx).await;
                        }

                        Event::ReactionAdded(ev) => {
                            let (room, message, user) = tokio::join!(
                                async { bot_for_hdlr.fetch_room(&ev.room_id).await.ok() },
                                async {
                                    bot_for_hdlr
                                        .fetch_message_raw(&ev.room_id, &ev.message_event_id)
                                        .await
                                        .ok()
                                },
                                async {
                                    if let Some(actor_id) = actor_id {
                                        bot_for_hdlr.fetch_user(actor_id).await.ok()
                                    } else {
                                        None
                                    }
                                }
                            );

                            ctx.room = room;
                            ctx.message = message;
                            ctx.user = user;

                            hdlr.reaction_added(ctx, &ev.emoji.clone()).await;
                        }

                        Event::ReactionRemoved(ev) => {
                            let (room, message, user) = tokio::join!(
                                async { bot_for_hdlr.fetch_room(&ev.room_id).await.ok() },
                                async {
                                    bot_for_hdlr
                                        .fetch_message_raw(&ev.room_id, &ev.message_event_id)
                                        .await
                                        .ok()
                                },
                                async {
                                    if let Some(actor_id) = actor_id {
                                        bot_for_hdlr.fetch_user(actor_id).await.ok()
                                    } else {
                                        None
                                    }
                                }
                            );

                            ctx.room = room;
                            ctx.message = message;
                            ctx.user = user;

                            hdlr.reaction_removed(ctx, &ev.emoji.clone()).await;
                        }

                        _ => {}
                    }
                });
            }
        });
        Ok(())
    }

    pub async fn fetch_room(&self, id: &str) -> CResult<Room> {
        let room = self
            .room_directory_service
            .get_room(GetRoomRequest {
                room_id: id.to_string(),
                ..Default::default()
            })
            .await?
            .into_owned()
            .room
            .into_option()
            .and_then(|r| r.room.into_option())
            .ok_or(ChurroError::ResourceNotFound("room"))?;
        Ok(room)
    }

    pub async fn fetch_user(&self, id: &str) -> CResult<User> {
        let user = self
            .user_service
            .get_user(GetUserRequest {
                target: Some(Target::UserId(id.to_string())),
                ..Default::default()
            })
            .await?
            .into_owned()
            .user
            .into_option()
            .and_then(|u| u.user.into_option())
            .ok_or(ChurroError::ResourceNotFound("user"))?;
        Ok(user)
    }

    pub async fn fetch_message_raw(&self, room_id: &str, event_id: &str) -> CResult<Message> {
        let message = self
            .message_service
            .get_message(GetMessageRequest {
                room_id: room_id.to_string(),
                event_id: event_id.to_string(),
                ..Default::default()
            })
            .await?
            .into_owned()
            .message
            .into_option()
            .ok_or(ChurroError::ResourceNotFound("message"))?;
        Ok(message)
    }

    pub async fn fetch_message(&self, room: &Room, event_id: &str) -> CResult<Message> {
        self.fetch_message_raw(&room.id, event_id).await
    }

    pub async fn send_message_raw(&self, room_id: &str, msg: &str) -> CResult {
        self.message_service
            .create_message(CreateMessageRequest {
                room_id: room_id.to_string(),
                body: msg.to_string(),
                ..Default::default()
            })
            .await?;
        Ok(())
    }

    pub async fn send_message(&self, room: &Room, msg: &str) -> CResult {
        self.send_message_raw(&room.id, msg).await
    }

    pub async fn reply_raw(
        &self,
        room_id: &str,
        id: &str,
        text: &str,
        send_to_channel: bool,
    ) -> CResult {
        self.message_service
            .create_message(CreateMessageRequest {
                room_id: room_id.to_string(),
                body: text.to_string(),
                in_reply_to: id.to_string(),
                also_send_to_channel: send_to_channel,
                ..Default::default()
            })
            .await?;
        Ok(())
    }

    pub async fn reply(&self, msg: &Message, text: &str, send_to_channel: bool) -> CResult {
        self.reply_raw(&msg.room_id, &msg.id, text, send_to_channel)
            .await
    }

    pub async fn reply_in_thread_raw(
        &self,
        room_id: &str,
        id: &str,
        text: &str,
        send_to_channel: bool,
    ) -> CResult {
        self.message_service
            .create_message(CreateMessageRequest {
                room_id: room_id.to_string(),
                body: text.to_string(),
                thread_root_event_id: id.to_string(),
                in_reply_to: id.to_string(),
                also_send_to_channel: send_to_channel,
                ..Default::default()
            })
            .await?;
        Ok(())
    }

    pub async fn reply_in_thread(
        &self,
        msg: &Message,
        text: &str,
        send_to_channel: bool,
    ) -> CResult {
        if msg.thread_root_event_id.is_empty() {
            self.reply_in_thread_raw(&msg.room_id, &msg.id, text, send_to_channel)
                .await
        } else {
            self.reply(msg, text, send_to_channel).await
        }
    }

    pub async fn send_in_thread_raw(
        &self,
        room_id: &str,
        root_id: &str,
        text: &str,
        send_to_channel: bool,
    ) -> CResult {
        self.message_service
            .create_message(CreateMessageRequest {
                room_id: room_id.to_string(),
                body: text.to_string(),
                thread_root_event_id: root_id.to_string(),
                also_send_to_channel: send_to_channel,
                ..Default::default()
            })
            .await?;
        Ok(())
    }

    pub async fn send_in_thread(
        &self,
        root_msg: &Message,
        text: &str,
        send_to_channel: bool,
    ) -> CResult {
        let root_id: &str = if root_msg.thread_root_event_id.is_empty() {
            &root_msg.id
        } else {
            &root_msg.thread_root_event_id
        };
        self.send_in_thread_raw(&root_msg.room_id, root_id, text, send_to_channel)
            .await
    }

    pub async fn set_status(&self, emoji: &str, text: &str) -> CResult {
        self.my_account_service
            .update_custom_status(UpdateCustomStatusRequest {
                emoji: emoji.to_string(),
                text: text.to_string(),
                ..Default::default()
            })
            .await?;
        Ok(())
    }

    pub async fn clear_status(&self) -> CResult {
        self.my_account_service
            .delete_custom_status(DeleteCustomStatusRequest {
                ..Default::default()
            })
            .await?;
        Ok(())
    }

    pub async fn join_room_raw(&self, room_id: &str) -> CResult {
        self.room_service
            .join_room(JoinRoomRequest {
                room_id: room_id.to_string(),
                ..Default::default()
            })
            .await?;
        Ok(())
    }

    pub async fn join_room(&self, room: &Room) -> CResult {
        self.join_room_raw(&room.id).await
    }

    pub async fn leave_room_raw(&self, room_id: &str) -> CResult {
        self.room_service
            .leave_room(LeaveRoomRequest {
                room_id: room_id.to_string(),
                ..Default::default()
            })
            .await?;
        Ok(())
    }

    pub async fn leave_room(&self, room: &Room) -> CResult {
        self.leave_room_raw(&room.id).await
    }
}
