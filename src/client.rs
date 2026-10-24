use crate::proto::chatto::admin::v1::*;
use crate::proto::chatto::api::v1::get_user_request::Target;
use crate::proto::chatto::api::v1::{GetRoomRequest, *};
use crate::proto::chatto::auth::v1::*;
use crate::proto::chatto::discovery::v1::*;
use crate::proto::chatto::realtime::v1::*;
use crate::{CResult, ChurroError};

use buffa::Message as _;
use connectrpc::Protocol;
use connectrpc::client::{ClientConfig, HttpClient};
use connectrpc::rustls::ClientConfig as TLSConfig;
use connectrpc::rustls::RootCertStore;
use futures_util::stream::StreamExt;
use futures_util::{SinkExt, Stream};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as WSMessage;

use std::pin::Pin;

/// The struct for Chatto clients.
#[derive(Clone)]
pub struct Client {
    token: String,
    server: String,
    secure: bool,
    // Auth Services
    pub external_identity_auth_service: ExternalIdentityAuthServiceClient<HttpClient>,
    pub server_setup_service: ServerSetupServiceClient<HttpClient>,
    pub push_subscription_cleanup_service: PushSubscriptionCleanupServiceClient<HttpClient>,
    // Discovery Services
    pub server_discovery_service: ServerDiscoveryServiceClient<HttpClient>,
    // Misc API Services
    pub asset_service: AssetServiceClient<HttpClient>,
    pub asset_upload_service: AssetUploadServiceClient<HttpClient>,
    pub bot_service: BotServiceClient<HttpClient>,
    pub permission_service: PermissionServiceClient<HttpClient>,
    pub message_service: MessageServiceClient<HttpClient>,
    pub message_search_service: MessageSearchServiceClient<HttpClient>,
    pub my_account_service: MyAccountServiceClient<HttpClient>,
    pub notification_service: NotificationServiceClient<HttpClient>,
    pub push_notification_service: PushNotificationServiceClient<HttpClient>,
    pub role_service: RoleServiceClient<HttpClient>,
    pub room_directory_service: RoomDirectoryServiceClient<HttpClient>,
    //  pub room_invitation_service: RoomInvitationServiceClient<HttpClient>,
    pub room_service: RoomServiceClient<HttpClient>,
    pub server_service: ServerServiceClient<HttpClient>,
    pub thread_service: ThreadServiceClient<HttpClient>,
    pub user_service: UserServiceClient<HttpClient>,
    pub viewer_service: ViewerServiceClient<HttpClient>,
    pub voice_call_service: VoiceCallServiceClient<HttpClient>,
    // Admin Services
    pub admin_invite_link_service: AdminInviteLinkServiceClient<HttpClient>,
    pub admin_oauth_client_service: AdminOAuthClientServiceClient<HttpClient>,
    pub admin_diagnostics_service: AdminDiagnosticsServiceClient<HttpClient>,
    pub admin_event_log_service: AdminEventLogServiceClient<HttpClient>,
    pub admin_permission_service: AdminPermissionServiceClient<HttpClient>,
    pub admin_role_service: AdminRoleServiceClient<HttpClient>,
    pub admin_room_layout_service: AdminRoomLayoutServiceClient<HttpClient>,
    pub admin_server_service: AdminServerServiceClient<HttpClient>,
    pub admin_user_service: AdminUserServiceClient<HttpClient>,
}

impl Client {
    fn make_client(
        conn: HttpClient,
        config: ClientConfig,
        token: &str,
        server: &str,
        secure: bool,
    ) -> Client {
        Client {
            token: token.to_string(),
            server: server.to_string(),
            secure,
            // Auth
            external_identity_auth_service: ExternalIdentityAuthServiceClient::new(
                conn.clone(),
                config.clone(),
            ),
            server_setup_service: ServerSetupServiceClient::new(conn.clone(), config.clone()),
            push_subscription_cleanup_service: PushSubscriptionCleanupServiceClient::new(
                conn.clone(),
                config.clone(),
            ),
            // Discovery
            server_discovery_service: ServerDiscoveryServiceClient::new(
                conn.clone(),
                config.clone(),
            ),
            // Misc API
            asset_service: AssetServiceClient::new(conn.clone(), config.clone()),
            asset_upload_service: AssetUploadServiceClient::new(conn.clone(), config.clone()),
            bot_service: BotServiceClient::new(conn.clone(), config.clone()),
            permission_service: PermissionServiceClient::new(conn.clone(), config.clone()),
            message_service: MessageServiceClient::new(conn.clone(), config.clone()),
            message_search_service: MessageSearchServiceClient::new(conn.clone(), config.clone()),
            my_account_service: MyAccountServiceClient::new(conn.clone(), config.clone()),
            notification_service: NotificationServiceClient::new(conn.clone(), config.clone()),
            push_notification_service: PushNotificationServiceClient::new(
                conn.clone(),
                config.clone(),
            ),
            role_service: RoleServiceClient::new(conn.clone(), config.clone()),
            room_directory_service: RoomDirectoryServiceClient::new(conn.clone(), config.clone()),
            // room_invitation_service: RoomInvitationServiceClient::new(conn.clone(), config.clone()),
            room_service: RoomServiceClient::new(conn.clone(), config.clone()),
            server_service: ServerServiceClient::new(conn.clone(), config.clone()),
            thread_service: ThreadServiceClient::new(conn.clone(), config.clone()),
            user_service: UserServiceClient::new(conn.clone(), config.clone()),
            viewer_service: ViewerServiceClient::new(conn.clone(), config.clone()),
            voice_call_service: VoiceCallServiceClient::new(conn.clone(), config.clone()),
            // Admin
            admin_invite_link_service: AdminInviteLinkServiceClient::new(
                conn.clone(),
                config.clone(),
            ),
            admin_oauth_client_service: AdminOAuthClientServiceClient::new(
                conn.clone(),
                config.clone(),
            ),
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
        }
    }

    /// Login to the Chatto server with a given token (bot or user).
    pub async fn with_token(token: &str, server: &str) -> CResult<Self> {
        let secure = if server.starts_with("https://") {
            true
        } else if server.starts_with("http://") {
            false
        } else {
            Err(ChurroError::NotAUrl)?
        };

        let conn = if secure {
            let mut roots = RootCertStore::empty();
            roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

            let tls_config = TLSConfig::builder()
                .with_root_certificates(roots)
                .with_no_client_auth();

            HttpClient::with_tls(tls_config.into())
        } else {
            HttpClient::plaintext()
        };

        let config = ClientConfig::new(format!("{}/api/connect", server).parse()?)
            .with_protocol(Protocol::Connect)
            .with_default_header("Authorization", format!("Bearer {}", token));

        let server = if secure {
            server.strip_prefix("https://").unwrap_or(server)
        } else {
            server.strip_prefix("http://").unwrap_or(server)
        };

        Ok(Self::make_client(conn, config, token, server, secure))
    }

    /// Start a stream of realtime events.
    pub async fn event_stream(&self) -> CResult<Pin<Box<dyn Stream<Item = RealtimeEvent> + '_>>> {
        let url = format!(
            "{}://{}/api/realtime",
            if self.secure { "wss" } else { "ws" },
            self.server
        );
        let (ws, _) = connect_async(url).await?;
        let (mut tx, rx) = ws.split();

        tx.send(WSMessage::Binary(
            RealtimeSubscribe {
                protocol_version: 4,
                bearer_token: Some(self.token.clone()),
                initial_state: buffa::EnumValue::Known(
                    RealtimeInitialState::REALTIME_INITIAL_STATE_LIVE_ONLY,
                ),
                ..Default::default()
            }
            .encode_to_bytes(),
        ))
        .await?;

        let stream = rx.filter_map(|m| async move {
            let Ok(m) = m else {
                return None;
            };
            let Ok(f) = RealtimeServerFrame::decode_from_slice(&m.into_data()) else {
                return None;
            };
            let Some(f) = f.frame else {
                return None;
            };
            let crate::proto::chatto::realtime::v1::realtime_server_frame::Frame::Event(e) = f
            else {
                return None;
            };
            Some(*e)
        });

        Ok(Box::pin(stream))
    }

    /// Fetch a user by their ID. Not cached.
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

    /// Fetch a message by its eventID. Not cached.
    pub async fn fetch_msg(&self, room_id: &str, id: &str) -> CResult<Message> {
        let msg = self
            .message_service
            .get_message(GetMessageRequest {
                room_id: room_id.to_string(),
                event_id: id.to_string(),
                ..Default::default()
            })
            .await?
            .into_owned()
            .message
            .into_option()
            .ok_or(ChurroError::ResourceNotFound("message"))?;
        Ok(msg)
    }

    /// Fetch a room by its roomID. Not cached.
    pub async fn fetch_room(&self, room_id: &str) -> CResult<Room> {
        let room = self
            .room_directory_service
            .get_room(GetRoomRequest {
                room_id: room_id.to_string(),
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
}
