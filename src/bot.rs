use connectrpc::Protocol;
use connectrpc::client::{ClientConfig, HttpClient};
use connectrpc::rustls::ClientConfig as TLSConfig;
use connectrpc::rustls::RootCertStore;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

use crate::proto::chatto::admin::v1::*;
use crate::proto::chatto::api::v1::*;
use crate::proto::chatto::auth::v1::*;
use crate::proto::chatto::discovery::v1::*;

use std::error::Error;
use std::fmt;
use std::fmt::Formatter;

pub struct Bot {
    token: String,
    user_id: String,
    username: String,
    conn: HttpClient,
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
    success: bool,
    token: String,
    user: LoginUser,
}

#[derive(Deserialize)]
struct LoginUser {
    id: String,
    login: String,
}

#[derive(Deserialize, Debug)]
struct LoginError {
    error: String,
}

impl Error for LoginError {}

impl fmt::Display for LoginError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.error)
    }
}

impl Bot {
    pub async fn login(
        username: &str,
        password: &str,
        server: &str,
    ) -> Result<Self, Box<dyn Error>> {
        let resp = Client::new()
            .post(format!("{}/auth/login", server))
            .json(&json!({
                "login": username,
                "password": password,
            }))
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await?;

        if !status.is_success() {
            return Err(Box::new(serde_json::from_str::<LoginError>(&body)?));
        }

        let result: LoginResult = serde_json::from_str(&body)?;

        let token = result.token;

        let mut roots = RootCertStore::empty();
        roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let tls_config = TLSConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();

        let conn = HttpClient::with_tls(tls_config.into());

        let config = ClientConfig::new(format!("{}/api/connect", server).parse()?)
            .with_protocol(Protocol::Connect)
            .with_default_header("Authorization", format!("Bearer {}", token.clone()));

        Ok(Bot {
            token,
            user_id: result.user.id,
            username: result.user.login,
            conn: conn.clone(),
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
}
