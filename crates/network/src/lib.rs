use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use sui_core::messages::ExecutionRequest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitTransactionRequest {
    pub transaction: ExecutionRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitTransactionResponse {
    pub accepted: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetObjectRequest {
    pub object_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetObjectResponse {
    pub found: bool,
    pub object: Option<serde_json::Value>,
}

pub struct NetworkServer {
    port: u16,
}

impl NetworkServer {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub async fn start<H>(self, handler: H) -> Result<()>
    where
        H: TransactionHandler + Clone + Send + Sync + 'static,
    {
        let app_state = AppState {
            handler: Arc::new(handler),
        };

        let app = Router::new()
            .route("/health", get(health_check))
            .route("/submit_transaction", post(submit_transaction))
            .route("/get_object", post(get_object))
            .with_state(app_state);

        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;
        println!("Network server listening on port {}", self.port);
        axum::serve(listener, app).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
pub trait TransactionHandler: Send + Sync {
    async fn handle_transaction(&self, request: ExecutionRequest) -> Result<SubmitTransactionResponse>;
    async fn get_object(&self, object_id: &str) -> Result<Option<serde_json::Value>>;
}

#[derive(Clone)]
struct AppState {
    handler: Arc<dyn TransactionHandler>,
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn submit_transaction(
    State(state): State<AppState>,
    Json(payload): Json<SubmitTransactionRequest>,
) -> Result<Json<SubmitTransactionResponse>, StatusCode> {
    match state.handler.handle_transaction(payload.transaction).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            eprintln!("Error handling transaction: {}", e);
            Ok(Json(SubmitTransactionResponse {
                accepted: false,
                message: format!("Error: {}", e),
            }))
        }
    }
}

async fn get_object(
    State(state): State<AppState>,
    Json(payload): Json<GetObjectRequest>,
) -> Result<Json<GetObjectResponse>, StatusCode> {
    match state.handler.get_object(&payload.object_id).await {
        Ok(Some(obj)) => Ok(Json(GetObjectResponse {
            found: true,
            object: Some(obj),
        })),
        Ok(None) => Ok(Json(GetObjectResponse {
            found: false,
            object: None,
        })),
        Err(e) => {
            eprintln!("Error getting object: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub struct NetworkClient {
    base_url: String,
    client: reqwest::Client,
}

impl NetworkClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn submit_transaction(&self, request: ExecutionRequest) -> Result<SubmitTransactionResponse> {
        let url = format!("{}/submit_transaction", self.base_url);
        let payload = SubmitTransactionRequest { transaction: request };
        let response = self.client.post(&url).json(&payload).send().await?;
        let result: SubmitTransactionResponse = response.json().await?;
        Ok(result)
    }

    pub async fn get_object(&self, object_id: &str) -> Result<GetObjectResponse> {
        let url = format!("{}/get_object", self.base_url);
        let payload = GetObjectRequest {
            object_id: object_id.to_string(),
        };
        let response = self.client.post(&url).json(&payload).send().await?;
        let result: GetObjectResponse = response.json().await?;
        Ok(result)
    }
}

