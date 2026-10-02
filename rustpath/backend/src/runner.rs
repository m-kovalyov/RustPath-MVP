use crate::{domain::Evaluation, error::AppError};
use async_trait::async_trait;
use serde_json::json;

#[async_trait]
pub trait CodeEvaluator: Send + Sync {
    async fn evaluate(&self, code: &str, tests: &str) -> Result<Evaluation, AppError>;
}
pub struct IsolatedRunner {
    pub client: reqwest::Client,
    pub url: String,
    pub secret: String,
}
#[async_trait]
impl CodeEvaluator for IsolatedRunner {
    async fn evaluate(&self, code: &str, tests: &str) -> Result<Evaluation, AppError> {
        let source = format!("{code}\n\n{tests}");
        let response = self
            .client
            .post(format!("{}/evaluate", self.url.trim_end_matches('/')))
            .bearer_auth(&self.secret)
            .json(&json!({"source": source}))
            .send()
            .await
            .map_err(|e| {
                tracing::warn!(error = %e, "runner unavailable");
                AppError::Runner
            })?;
        if !response.status().is_success() {
            return Err(AppError::Runner);
        }
        response
            .json::<Evaluation>()
            .await
            .map_err(|_| AppError::Runner)
    }
}
