use anyhow::Result;
use argon2::{
    Argon2, PasswordHasher,
    password_hash::{Encoding, SaltString},
};
use argon2::{PasswordHash, PasswordVerifier, password_hash::rand_core::OsRng};
use axum::extract::{FromRequestParts, Path};
use axum::{
    Extension, Json, Router,
    extract::{Query, Request, State, rejection::JsonRejection},
    http::StatusCode,
    middleware::{Next, from_fn_with_state},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use axum::{extract::Multipart, http::HeaderMap};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use std::{
    cmp::min,
    collections::HashSet,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};

use dashmap::DashMap;
use jwt_simple::prelude::*;
use jwt_simple::{
    claims::Claims,
    common::VerificationOptions,
    prelude::{Duration, Ed25519KeyPair, Ed25519PublicKey, EdDSAKeyPairLike},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha1::Digest;
use thiserror::Error;
use tokio::{fs, net::TcpListener};
use tracing::{info, instrument, level_filters::LevelFilter, warn};
use tracing_subscriber::{
    Layer as _,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
const JWT_DURATION: u64 = 60 * 60 * 24 * 7;
const JWT_ISS: &str = "chat_server";
const JWT_AUD: &str = "chat_client";
fn default_limit() -> u32 {
    10
}
#[derive(Deserialize, Debug)]
struct UserFilter {
    #[serde(default = "default_limit")]
    limit: u32,
    ordering: Option<String>,
}
#[derive(Serialize)]
struct JoinOutput {
    message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct JwtUser {
    id: u32,
    name: String,
    email: String,
}

#[derive(Deserialize)]
struct SigninRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct SigninOutput {
    token: String,
}

#[derive(Serialize, Clone)]
struct Message {
    chat_id: u32,
    sender_id: u32,
    content: String,
    files: Vec<PathBuf>,
    date: DateTime<Utc>,
}
#[derive(Serialize, Deserialize)]
struct MessageRequest {
    content: String,
    files: Vec<PathBuf>,
}

#[derive(Deserialize, Serialize, Eq, Hash, PartialEq, Clone, Debug)]
struct CreateChatRequest {
    chat_id: u32,
}

#[derive(Deserialize, Debug)]
struct JoinChatRequest {
    chat_id: u32,
}
#[derive(Clone)]
struct EncodingKey(Ed25519KeyPair);
#[derive(Clone)]
struct DecodingKey(Ed25519PublicKey);

#[derive(Deserialize, Serialize)]
struct ChatFile {
    chat_id: u32,
    sh1_hash: String,
    ext: String,
}

#[derive(Clone)]
struct AppState {
    data: Arc<DashMap<u32, User>>,
    idx: Arc<AtomicU32>,
    ek: EncodingKey,
    pk: DecodingKey,
    chats: Arc<DashMap<u32, Vec<u32>>>,
    Messages: Arc<DashMap<u32, DashMap<u32, Vec<Message>>>>,
    base_dir: PathBuf,
}

#[derive(Serialize, Debug)]
struct SignupOutput {
    token: String,
    user: User,
}
#[derive(Deserialize, Debug)]
struct UserCreateRequest {
    name: String,
    email: String,
    gender: Option<String>,
    #[serde(default)]
    teenager: bool,
    password: String,
}

#[derive(Serialize, Debug, Clone)]
struct User {
    #[serde(default)]
    id: u32,
    name: String,
    email: String,
    gender: String,
    #[serde(default)]
    teenager: bool,
    #[serde(skip_serializing)]
    hash_password: String,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    CreateUserError(String),

    #[error("{0}")]
    HashPasswordError(String),
    #[error("{0}")]
    JwtSignError(String),
    #[error("{0}")]
    JwtLoadError(String),
    #[error("{0}")]
    JwtVerifyError(String),
    #[error("{0}")]
    UserNotFound(String),
    #[error("{0}")]
    PasswordVerifyError(String),
    #[error("{0}")]
    CreateChatError(String),
    #[error("{0}")]
    JoinChatError(String),
    #[error("{0}")]
    UploadFileError(String),
    #[error("{0}")]
    FileReadError(#[from] std::io::Error),
    #[error("{0}")]
    FileNotFound(String),
}
#[tokio::main]
async fn main() -> Result<()> {
    let file_appender = tracing_appender::rolling::daily("tmp", "http_post.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let console = fmt::layer()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .pretty()
        .with_filter(LevelFilter::DEBUG);

    let file = fmt::layer()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .pretty()
        .with_writer(non_blocking)
        .with_filter(LevelFilter::DEBUG);

    tracing_subscriber::registry()
        .with(console)
        .with(file)
        .init();
    let ek = EncodingKey::load().await?;
    let pk = DecodingKey::load().await?;
    let base_dir: PathBuf = "/tmp/chat_server".into();
    fs::create_dir_all(&base_dir).await?;
    let state = AppState {
        data: Arc::new(DashMap::new()),
        idx: Arc::new(AtomicU32::new(0)),
        ek,
        pk,
        chats: Arc::new(DashMap::new()),
        base_dir,
        Messages: Arc::new(DashMap::new()),
    };
    let addr = "127.0.0.1:8080";
    info!("Listening on http://{addr}");

    let listener = TcpListener::bind(addr).await?;
    info!("TcpListener initialized on http://{addr}");
    let message_api = Router::new()
        .route("/upload_file/{id}", post(upload_file_handler))
        .route("/send_message/{id}", post(send_message_handler))
        .layer(from_fn_with_state(
            state.clone(),
            message_verify_jwt_token_middleware,
        ));

    let chat_api = Router::new()
        .route("/create_chat", post(create_chat_handler))
        .route("/join_chat", post(join_chat_handler))
        .nest("/message", message_api)
        .route("/message/files/{id}/{*path}", get(file_handler))
        .layer(from_fn_with_state(
            state.clone(),
            chat_verify_jwt_token_middleware,
        ));
    //

    let app = Router::new()
        .route("/users/list", get(list_users_handler))
        .route("/users/list_with_query", get(query_users_handler))
        .route("/users/create", post(signup_handler))
        .route("/users/signin", post(signin_handler))
        .nest("/users", chat_api)
        .with_state(state);

    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

#[instrument(skip(state))]
async fn list_users_handler(State(state): State<AppState>) -> impl IntoResponse {
    let users = state
        .data
        .iter()
        .map(|user| user.value().clone())
        .collect::<Vec<_>>();
    Json(users)
}

#[instrument(skip(state, payload))]
async fn signup_handler(
    State(state): State<AppState>,
    payload: Result<Json<UserCreateRequest>, JsonRejection>,
) -> Result<impl IntoResponse, AppError> {
    let Json(payload) = match payload {
        Ok(v) => v,
        Err(e) => return Err(AppError::CreateUserError(format!("invalid json: {e}"))),
    };

    let Some(gender) = payload.gender else {
        return Err(AppError::CreateUserError("gender is required".into()));
    };
    if state
        .data
        .iter()
        .any(|user| user.value().email == payload.email)
    {
        return Err(AppError::CreateUserError("email already exists".into()));
    }
    let hashed_password = hash_password_with_argon2(&payload.password)?;
    //find whether the user with the same email already exists

    let id = state.idx.fetch_add(1, Ordering::SeqCst) + 1;
    let user = User {
        id,
        name: payload.name,
        email: payload.email,
        gender,
        teenager: payload.teenager,
        hash_password: hashed_password,
    };
    state.data.insert(id, user.clone());
    let token = state.ek.sign(JwtUser {
        id: user.id,
        name: user.name.clone(),
        email: user.email.clone(),
    })?;
    Ok((StatusCode::CREATED, Json(SignupOutput { token, user })))
}

#[instrument(skip(state))]
async fn query_users_handler(
    State(state): State<AppState>,
    Query(query): Query<UserFilter>,
) -> impl IntoResponse {
    let limit = min(query.limit, state.idx.load(Ordering::SeqCst));

    let ordering = query.ordering.unwrap_or_else(|| "asc".to_string());
    let mut users = state
        .data
        .iter()
        .map(|user| user.value().clone())
        .collect::<Vec<_>>();
    if ordering == "asc" {
        users.sort_by(|a, b| a.id.cmp(&b.id));
    } else {
        users.sort_by(|a, b| b.id.cmp(&a.id));
    }
    users.truncate(limit as usize);
    Json(users)
}

#[instrument(skip(state, signin_request))]
async fn signin_handler(
    State(state): State<AppState>,
    Json(signin_request): Json<SigninRequest>,
) -> Result<impl IntoResponse, AppError> {
    //first, find if the user with the email exists
    let user = match state
        .data
        .iter()
        .find(|user| user.value().email == signin_request.email.clone())
    {
        Some(user) => user.value().clone(),
        None => return Err(AppError::UserNotFound("email not found".into())),
    };
    //second, verify the password with argon2 and  the hash_password stored in the state
    if !verify_password_with_argon2(&signin_request.password, &user.hash_password) {
        return Err(AppError::PasswordVerifyError("invalid password".into()));
    }
    //third ,return the jwt
    let token = state.ek.sign(JwtUser {
        id: user.id,
        name: user.name.clone(),
        email: user.email.clone(),
    })?;
    Ok((StatusCode::OK, Json(SigninOutput { token })))
}

#[instrument(skip(state))]
async fn create_chat_handler(
    State(state): State<AppState>,
    Extension(user): Extension<JwtUser>,
    Json(chat): Json<CreateChatRequest>,
) -> Result<impl IntoResponse, AppError> {
    let state_chats = state
        .chats
        .iter()
        .map(|chat| chat.key().clone())
        .collect::<Vec<_>>();
    if state_chats.iter().any(|c| *c == chat.chat_id) {
        return Err(AppError::CreateChatError("chat id already exists".into()));
    }
    state.chats.insert(chat.chat_id, vec![user.id]);
    Ok((StatusCode::CREATED, Json(chat)))
}

#[instrument(skip(state))]
async fn join_chat_handler(
    State(state): State<AppState>,
    Extension(user): Extension<JwtUser>,
    Json(join_chat): Json<JoinChatRequest>,
) -> Result<impl IntoResponse, AppError> {
    let chat = join_chat.chat_id;
    let chats = state
        .chats
        .iter()
        .map(|chat| chat.key().clone())
        .collect::<Vec<_>>();
    let Some(chat) = chats.into_iter().find(|c| *c == chat) else {
        return Err(AppError::JoinChatError("chat id not found".into()));
    };
    state.chats.entry(chat).and_modify(|users| {
        if !users.contains(&user.id) {
            users.push(user.id);
        }
    });
    Ok((
        StatusCode::OK,
        Json(JoinOutput {
            message: format!("join the chat {} successfully", join_chat.chat_id),
        }),
    ))
}

async fn chat_verify_jwt_token_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let (mut parts, body) = req.into_parts();
    let req =
        match TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &state).await {
            Ok(TypedHeader(Authorization(bearer))) => {
                let token = bearer.token();
                match state.pk.verify(token) {
                    Ok(user) => {
                        let mut req = Request::from_parts(parts, body);
                        req.extensions_mut().insert(user);
                        req
                    }
                    Err(e) => {
                        return (StatusCode::UNAUTHORIZED, format!("verify jwt failed: {e}"))
                            .into_response();
                    }
                }
            }
            Err(e) => {
                let msg = format!("parse Authorization header failed: {}", e);
                warn!(msg);
                return (StatusCode::UNAUTHORIZED, msg).into_response();
            }
        };
    next.run(req).await
}

//upload the file to the base_dir with the name of {file}/{chat_id}/{sha1_hash}.{ext}
async fn upload_file_handler(
    Extension(user): Extension<JwtUser>,
    Extension(chat_id): Extension<u32>,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut files: Vec<PathBuf> = vec![];
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::UploadFileError(e.to_string()))?
    {
        let filename = field.file_name().map(|s| s.to_string()).unwrap();
        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::UploadFileError(e.to_string()))?;
        let chat_file = ChatFile {
            chat_id,
            sh1_hash: format!("{:x}", sha1::Sha1::digest(&data)),
            ext: filename.split(".").last().unwrap_or("txt").to_string(),
        };
        let (path1, path2) = chat_file.sh1_hash.split_at(3);
        let (path2, path3) = path2.split_at(3);
        let path: PathBuf = format!(
            "{}/file/{}/{}/{}.{}",
            chat_id, path1, path2, path3, chat_file.ext
        )
        .into();
        let fullpath = state.base_dir.join(&path);
        info!("{} upload file to {}", user.name, fullpath.display());
        fs::create_dir_all(fullpath.parent().unwrap())
            .await
            .map_err(|e| AppError::UploadFileError(e.to_string()))?;
        fs::write(&fullpath, data)
            .await
            .map_err(|e| AppError::UploadFileError(e.to_string()))?;
        files.push(path);
    }
    Ok(Json(files))
}

//verify if the user is authorized to send message in the chat, which means the user has joined the chat
//and then show the file of the message in the chat
async fn file_handler(
    Path((chat_id, file_path)): Path<(u32, String)>,
    State(state): State<AppState>,
    Extension(user): Extension<JwtUser>,
) -> Result<impl IntoResponse, AppError> {
    let path_dir = state.base_dir.join(format!("{}/file", chat_id));
    info!("{}", path_dir.display());
    if !path_dir.exists() {
        return Err(AppError::UserNotFound("chat not found".to_string()));
    }
    let full_path = path_dir.join(file_path);
    if !full_path.exists() {
        return Err(AppError::UserNotFound("file not found".to_string()));
    }
    info!("{} download file {}", user.name, full_path.display());
    let mime = mime_guess::from_path(&full_path).first_or_octet_stream();
    let body = fs::read(&full_path).await?;
    let mut header = HeaderMap::new();
    header.insert("content-type", mime.to_string().parse().unwrap());
    Ok((StatusCode::OK, header, body))
}

//send message in the chat, which means the user has joined the chat and the file of the message is uploaded successfully
async fn send_message_handler(
    Extension(chat_id): Extension<u32>,
    Extension(user): Extension<JwtUser>,
    State(state): State<AppState>,
    Json(message_request): Json<MessageRequest>,
) -> Result<impl IntoResponse, AppError> {
    let content = message_request.content;
    if content.is_empty() {
        return Err(AppError::UserNotFound("content should not be empty".to_string()));
    }
    let is_member = state
        .chats
        .get(&chat_id)
        .map(|members| members.contains(&user.id))
        .unwrap_or(false);
    if !is_member {
        return Err(AppError::UserNotFound("user has not joined the chat".to_string()));
    }
    let files = message_request.files;
    for file in &files {
        if !file.exists() {
            return Err(AppError::FileNotFound(format!("file {} not found", file.display())));
        }
    }

    /*
    struct Message {
    chat_id: u32,
    sender_id: u32,
    content: String,
    files: Vec<ChatFile>,
    data: DateTime<Utc>,
    */
    let msg = Message {
        chat_id,
        sender_id: user.id,
        content,
        files,
        date: Utc::now(),
    };
    let mut chatmessages = state.Messages.entry(chat_id).or_insert(DashMap::new());
    if let Some(mut messages) = chatmessages.get_mut(&user.id) {
        messages.push(msg.clone());
    }
    let mut header = HeaderMap::new();
    header.insert("content-type", "application/json".parse().unwrap());
    Ok((StatusCode::OK, header, Json(msg)))
}

//verify if the user is authorized to send message in the chat, which means the user has joined the chat
async fn message_verify_jwt_token_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let (mut parts, body) = req.into_parts();
    //get the chat_id from the path

    //verify if the user has joined the chat
    let Path(chat_id) = Path::<u32>::from_request_parts(&mut parts, &state)
        .await
        .unwrap();

    let user = parts.extensions.get::<JwtUser>().unwrap();

    let is_member = state
        .chats
        .get(&chat_id)
        .map(|members| members.contains(&user.id))
        .unwrap_or(false);
    if !is_member {
        return (StatusCode::UNAUTHORIZED, "user has not joined the chat").into_response();
    }

    let mut req = Request::from_parts(parts, body);
    req.extensions_mut().insert(chat_id);
    next.run(req).await
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            Self::CreateUserError(_) => StatusCode::BAD_REQUEST,
            Self::HashPasswordError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::JwtSignError(_) => StatusCode::BAD_REQUEST,
            Self::JwtLoadError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::JwtVerifyError(_) => StatusCode::UNAUTHORIZED,
            Self::UserNotFound(_) => StatusCode::NOT_FOUND,
            Self::PasswordVerifyError(_) => StatusCode::UNAUTHORIZED,
            Self::CreateChatError(_) => StatusCode::BAD_REQUEST,
            Self::JoinChatError(_) => StatusCode::BAD_REQUEST,
            Self::UploadFileError(_) => StatusCode::BAD_REQUEST,
            Self::FileReadError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::FileNotFound(_) => StatusCode::NOT_FOUND,
        };
        (status, Json(serde_json::json!({"error": self.to_string()}))).into_response()
    }
}

fn hash_password_with_argon2(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::HashPasswordError(format!("hash password failed: {e}")))?
        .to_string();
    Ok(password_hash)
}

fn verify_password_with_argon2(password: &str, password_hash: &str) -> bool {
    let argon2 = Argon2::default();
    let parsed = match PasswordHash::new(password_hash) {
        Ok(p) => p,
        Err(_) => return false,
    };

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}
impl EncodingKey {
    async fn load() -> Result<Self, AppError> {
        let ek_pem = fs::read_to_string("encoding.pem")
            .await
            .map_err(|e| AppError::JwtLoadError(format!("read encoding key failed: {e}")))?;
        Ok(Self(Ed25519KeyPair::from_pem(&ek_pem).map_err(|e| {
            AppError::JwtLoadError(format!("parse encoding key failed: {e}"))
        })?))
    }
    fn sign(&self, jwt_user: JwtUser) -> Result<String, AppError> {
        let claims = Claims::with_custom_claims(jwt_user, Duration::from_secs(JWT_DURATION))
            .with_issuer(JWT_ISS)
            .with_audience(JWT_AUD);
        Ok(self
            .0
            .sign(claims)
            .map_err(|e| AppError::JwtSignError(format!("sign jwt failed: {e}")))?)
    }
}

impl DecodingKey {
    async fn load() -> Result<Self, AppError> {
        let pk_pem = fs::read_to_string("decoding.pem")
            .await
            .map_err(|e| AppError::JwtLoadError(format!("read decoding key failed: {e}")))?;
        Ok(Self(Ed25519PublicKey::from_pem(&pk_pem).map_err(|e| {
            AppError::JwtLoadError(format!("parse decoding key failed: {e}"))
        })?))
    }
    fn verify(&self, token: &str) -> Result<JwtUser, AppError> {
        let mut opts = VerificationOptions::default();
        opts.allowed_issuers = Some(HashSet::from_strings(&[JWT_ISS]));
        opts.allowed_audiences = Some(HashSet::from_strings(&[JWT_AUD]));
        let claims = self
            .0
            .verify_token::<JwtUser>(token, Some(opts))
            .map_err(|e| AppError::JwtVerifyError(format!("verify jwt failed: {e}")))?;
        Ok(claims.custom)
    }
}
