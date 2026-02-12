use std::{
    cmp::min,
    collections::HashSet,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};

use anyhow::Result;
use argon2::{PasswordHash, PasswordVerifier, password_hash::rand_core::OsRng};
use argon2::{
    Argon2, PasswordHasher,
    password_hash::{Encoding, SaltString},
};
use axum::{
    Json, Router,
    extract::{Query, State, rejection::JsonRejection},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use dashmap::DashMap;
use jwt_simple::prelude::*;
use jwt_simple::{
    claims::Claims,
    common::VerificationOptions,
    prelude::{Duration, Ed25519KeyPair, Ed25519PublicKey, EdDSAKeyPairLike},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{fs, net::TcpListener};
use tracing::{info, instrument, level_filters::LevelFilter};
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

#[derive(Serialize, Deserialize)]
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
#[derive(Clone)]
struct EncodingKey(Ed25519KeyPair);
#[derive(Clone)]
struct DecodingKey(Ed25519PublicKey);

#[derive(Clone)]
struct AppState {
    data: Arc<DashMap<u32, User>>,
    idx: Arc<AtomicU32>,
    ek: EncodingKey,
    pk: DecodingKey,
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
    let state = AppState {
        data: Arc::new(DashMap::new()),
        idx: Arc::new(AtomicU32::new(0)),
        ek,
        pk,
    };
    let addr = "127.0.0.1:8080";
    info!("Listening on http://{addr}");

    let listener = TcpListener::bind(addr).await?;
    info!("TcpListener initialized on http://{addr}");
    let app = Router::new()
        .route("/users/list", get(list_users_handler))
        .route("/users/list_with_query", get(query_users_handler))
        .route("/users/create", post(signup_handler))
        .route("/users/signin", post(signin_handler))
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
    let user = match state.data.iter().find(|user| user.value().email == signin_request.email.clone()) {
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
