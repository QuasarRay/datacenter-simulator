//! NCP control-plane profile: worker objects do not own operating-system processes.
//! Preserve binding and eviction below; deny runtime operations before any upgrade.
use crate::{middleware::AuthContext, state::ApiServerState};
use axum::{
    body::Body,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use rusternetes_common::{authz::{Decision, RequestAttributes}, Error, Result};
use rusternetes_storage::Storage;
use std::sync::Arc;
use tracing::info;

fn unavailable() -> Response {
    (StatusCode::NOT_IMPLEMENTED, Json(serde_json::json!({
        "apiVersion": "v1", "kind": "Status", "status": "Failure",
        "reason": "NotImplemented", "code": 501,
        "message": "NCP KWOK workers have no workload processes; use the separately authorized Incus workload API"
    }))).into_response()
}

async fn runtime_unavailable(
    state: Arc<ApiServerState>, auth: AuthContext,
    namespace: String, name: String, verb: &str, subresource: &str,
) -> Result<Response> {
    let attrs = RequestAttributes::new(auth.user, verb, "pods")
        .with_namespace(&namespace).with_name(&name).with_subresource(subresource);
    match state.authorizer.authorize(&attrs).await? {
        Decision::Allow => {},
        Decision::Deny(reason) => return Err(Error::Forbidden(reason)),
    }
    let key = rusternetes_storage::build_key("pods", Some(&namespace), &name);
    let _: rusternetes_common::resources::Pod = state.storage.get(&key).await?;
    Ok(unavailable())
}

macro_rules! unavailable_handler {
    ($name:ident, $verb:literal, $subresource:literal) => {
        pub async fn $name(
            State(state): State<Arc<ApiServerState>>,
            Extension(auth): Extension<AuthContext>,
            Path((namespace, name)): Path<(String, String)>,
        ) -> Result<Response> {
            runtime_unavailable(state, auth, namespace, name, $verb, $subresource).await
        }
    };
}
unavailable_handler!(get_logs, "get", "log");
unavailable_handler!(exec, "create", "exec");
unavailable_handler!(attach, "create", "attach");
unavailable_handler!(portforward, "create", "portforward");

#[cfg(test)]
mod ncp_tests {
    #[tokio::test]
    async fn unavailable_is_failure_and_never_a_synthetic_log() {
        let response = super::unavailable();
        assert_eq!(response.status(), 501);
        let body = axum::body::to_bytes(response.into_body(), 4096).await.unwrap();
        let status: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(status["status"], "Failure");
        assert_eq!(status["code"], 501);
    }
}

