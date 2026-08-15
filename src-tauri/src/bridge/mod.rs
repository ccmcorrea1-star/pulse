//! Fronteira IPC pública do Pulse.
//!
//! Este módulo contém somente DTOs, validação, commands de leitura e emissão
//! de eventos redigidos. Ele não conhece SQL, sockets, keyring ou os slots
//! internos do runtime.

use std::error::Error;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Runtime, State, WebviewWindow};

use crate::domain::DOMAIN_MODEL_VERSION;
use crate::runtime::{RuntimeError, RuntimePhase, RuntimeSnapshot, RuntimeState};

pub const BRIDGE_CONTRACT_VERSION: u16 = 1;
pub const BRIDGE_STATUS_EVENT: &str = "pulse:bridge:status";
pub const DOMAIN_EVENT_EVENT: &str = "pulse:domain:event";
pub const SNAPSHOT_INVALIDATED_EVENT: &str = "pulse:domain:snapshot-invalidated";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BridgeRequest<T> {
    pub bridge_contract_version: u16,
    pub request_id: String,
    pub payload: T,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmptyPayload {}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BridgeReadStatus {
    Success,
    Stale,
    Offline,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BridgeReadResponse<T> {
    pub bridge_contract_version: u16,
    pub request_id: String,
    pub status: BridgeReadStatus,
    pub generated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BridgeMode {
    Tauri,
    WebPreview,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PublicRuntimePhase {
    Created,
    Starting,
    Partial,
    Ready,
    Failed,
    Stopping,
    Stopped,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProductState {
    NotConfigured,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BridgeInfo {
    pub mode: BridgeMode,
    pub model_version: u16,
    pub runtime_phase: PublicRuntimePhase,
    pub product_commands_available: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BridgeSnapshot {
    pub runtime_phase: PublicRuntimePhase,
    pub product_state: ProductState,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BridgeEvent<T> {
    pub bridge_contract_version: u16,
    pub stream_id: String,
    pub sequence: u64,
    pub event_id: String,
    pub emitted_at: String,
    pub model_version: u16,
    pub payload: T,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BridgeStatusPayload {
    pub runtime_phase: PublicRuntimePhase,
    pub product_commands_available: bool,
}

pub type BridgeStatusEvent = BridgeEvent<BridgeStatusPayload>;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BridgeErrorCode {
    InvalidRequest,
    UnsupportedContractVersion,
    RuntimeNotReady,
    NotFound,
    AlreadyResolved,
    TrustRequired,
    CapabilityDenied,
    PeerOffline,
    TransportUnavailable,
    StorageUnavailable,
    Timeout,
    Canceled,
    Conflict,
    Internal,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BridgeError {
    pub bridge_contract_version: u16,
    pub request_id: String,
    pub code: BridgeErrorCode,
    pub retryable: bool,
    pub message_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
}

impl BridgeError {
    fn new(
        request_id: impl Into<String>,
        code: BridgeErrorCode,
        retryable: bool,
        message_key: &'static str,
        reason_code: Option<&'static str>,
    ) -> Self {
        Self {
            bridge_contract_version: BRIDGE_CONTRACT_VERSION,
            request_id: request_id.into(),
            code,
            retryable,
            message_key: message_key.to_owned(),
            reason_code: reason_code.map(str::to_owned),
        }
    }
}

impl Display for BridgeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "bridge command failed: {}", self.message_key)
    }
}

impl Error for BridgeError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BridgeEmitError;

impl Display for BridgeEmitError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("bridge status event emission failed")
    }
}

impl Error for BridgeEmitError {}

#[tauri::command]
pub fn bridge_get_info(
    window: WebviewWindow,
    request: BridgeRequest<EmptyPayload>,
    state: State<'_, RuntimeState>,
) -> Result<BridgeReadResponse<BridgeInfo>, BridgeError> {
    validate_request(&request)?;
    validate_origin(&window, &request.request_id)?;
    let snapshot = state
        .snapshot()
        .map_err(|error| runtime_error(&request.request_id, error))?;
    Ok(BridgeReadResponse {
        bridge_contract_version: BRIDGE_CONTRACT_VERSION,
        request_id: request.request_id,
        status: BridgeReadStatus::Success,
        generated_at: utc_timestamp(),
        observed_at: None,
        data: Some(BridgeInfo {
            mode: BridgeMode::Tauri,
            model_version: DOMAIN_MODEL_VERSION,
            runtime_phase: public_runtime_phase(snapshot.phase),
            product_commands_available: false,
        }),
    })
}

#[tauri::command]
pub fn bridge_get_snapshot(
    window: WebviewWindow,
    request: BridgeRequest<EmptyPayload>,
    state: State<'_, RuntimeState>,
) -> Result<BridgeReadResponse<BridgeSnapshot>, BridgeError> {
    validate_request(&request)?;
    validate_origin(&window, &request.request_id)?;
    let snapshot = state
        .snapshot()
        .map_err(|error| runtime_error(&request.request_id, error))?;
    Ok(BridgeReadResponse {
        bridge_contract_version: BRIDGE_CONTRACT_VERSION,
        request_id: request.request_id,
        status: BridgeReadStatus::Offline,
        generated_at: utc_timestamp(),
        observed_at: None,
        data: Some(BridgeSnapshot {
            runtime_phase: public_runtime_phase(snapshot.phase),
            product_state: ProductState::NotConfigured,
        }),
    })
}

pub fn emit_bridge_status<R: Runtime>(
    app: &AppHandle<R>,
    snapshot: &RuntimeSnapshot,
) -> Result<(), BridgeEmitError> {
    app.emit(BRIDGE_STATUS_EVENT, bridge_status_event(snapshot))
        .map_err(|_| BridgeEmitError)
}

pub fn bridge_status_event(snapshot: &RuntimeSnapshot) -> BridgeStatusEvent {
    let emitted_at = utc_timestamp();
    let stream_nonce = utc_epoch_seconds();
    BridgeEvent {
        bridge_contract_version: BRIDGE_CONTRACT_VERSION,
        stream_id: format!("runtime-{stream_nonce}-{}", std::process::id()),
        sequence: 1,
        event_id: format!("bridge-status-{stream_nonce}"),
        emitted_at,
        model_version: DOMAIN_MODEL_VERSION,
        payload: BridgeStatusPayload {
            runtime_phase: public_runtime_phase(snapshot.phase),
            product_commands_available: false,
        },
    }
}

pub fn validate_request<T>(request: &BridgeRequest<T>) -> Result<(), BridgeError> {
    let request_id_is_valid = is_valid_request_id(&request.request_id);
    let request_id = safe_request_id(&request.request_id);
    if request.bridge_contract_version != BRIDGE_CONTRACT_VERSION {
        return Err(BridgeError::new(
            request_id,
            BridgeErrorCode::UnsupportedContractVersion,
            false,
            "bridge.unsupportedContractVersion",
            Some("unsupported-contract-version"),
        ));
    }
    if !request_id_is_valid {
        return Err(BridgeError::new(
            request_id,
            BridgeErrorCode::InvalidRequest,
            false,
            "bridge.invalidRequest",
            Some("invalid-request-id"),
        ));
    }
    Ok(())
}

fn validate_origin(window: &WebviewWindow, request_id: &str) -> Result<(), BridgeError> {
    if window.label() == "main" {
        return Ok(());
    }

    Err(BridgeError::new(
        request_id,
        BridgeErrorCode::InvalidRequest,
        false,
        "bridge.invalidRequest",
        Some("unsupported-window"),
    ))
}

fn runtime_error(request_id: &str, _: RuntimeError) -> BridgeError {
    BridgeError::new(
        request_id,
        BridgeErrorCode::RuntimeNotReady,
        true,
        "bridge.runtimeNotReady",
        Some("runtime-snapshot-unavailable"),
    )
}

fn safe_request_id(value: &str) -> String {
    if is_valid_request_id(value) {
        value.to_owned()
    } else {
        "invalid-request".to_owned()
    }
}

fn is_valid_request_id(value: &str) -> bool {
    (1..=128).contains(&value.len())
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "._-:".contains(character))
}

fn public_runtime_phase(phase: RuntimePhase) -> PublicRuntimePhase {
    match phase {
        RuntimePhase::Created => PublicRuntimePhase::Created,
        RuntimePhase::Starting => PublicRuntimePhase::Starting,
        RuntimePhase::Partial => PublicRuntimePhase::Partial,
        RuntimePhase::Ready => PublicRuntimePhase::Ready,
        RuntimePhase::Failed => PublicRuntimePhase::Failed,
        RuntimePhase::Stopping => PublicRuntimePhase::Stopping,
        RuntimePhase::Stopped => PublicRuntimePhase::Stopped,
    }
}

fn utc_epoch_seconds() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_owned())
}

fn utc_timestamp() -> String {
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let seconds = duration.as_secs() as i64;
    let days = seconds / 86_400;
    let seconds_in_day = seconds % 86_400;
    let (year, month, day) = civil_date_from_days(days);
    let hour = seconds_in_day / 3_600;
    let minute = (seconds_in_day % 3_600) / 60;
    let second = seconds_in_day % 60;

    format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{:03}Z",
        duration.subsec_millis()
    )
}

fn civil_date_from_days(days_since_epoch: i64) -> (i64, i64, i64) {
    let shifted_days = days_since_epoch + 719_468;
    let era = if shifted_days >= 0 {
        shifted_days
    } else {
        shifted_days - 146_096
    } / 146_097;
    let day_of_era = shifted_days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{RuntimePhase, RuntimeSnapshot};

    fn partial_snapshot() -> RuntimeSnapshot {
        RuntimeSnapshot {
            phase: RuntimePhase::Partial,
            services: Vec::new(),
        }
    }

    #[test]
    fn request_and_response_use_camel_case_and_reject_unknown_fields() {
        let request: BridgeRequest<EmptyPayload> = serde_json::from_str(
            r#"{"bridgeContractVersion":1,"requestId":"request-1","payload":{}}"#,
        )
        .expect("valid request should deserialize");
        validate_request(&request).expect("valid request should pass validation");

        let error = serde_json::from_str::<BridgeRequest<EmptyPayload>>(
            r#"{"bridgeContractVersion":1,"requestId":"request-1","payload":{},"extra":true}"#,
        );
        assert!(error.is_err());

        let response = BridgeReadResponse {
            bridge_contract_version: BRIDGE_CONTRACT_VERSION,
            request_id: "request-1".to_owned(),
            status: BridgeReadStatus::Offline,
            generated_at: "1".to_owned(),
            observed_at: None,
            data: Some(BridgeSnapshot {
                runtime_phase: PublicRuntimePhase::Partial,
                product_state: ProductState::NotConfigured,
            }),
        };
        let json = serde_json::to_string(&response).expect("response should serialize");
        assert!(json.contains("bridgeContractVersion"));
        assert!(json.contains("productState"));
        assert!(!json.contains("Mutex"));
    }

    #[test]
    fn invalid_versions_and_ids_are_closed_errors() {
        let invalid_version = BridgeRequest {
            bridge_contract_version: 2,
            request_id: "request-1".to_owned(),
            payload: EmptyPayload::default(),
        };
        let error = validate_request(&invalid_version).expect_err("version should be rejected");
        assert_eq!(error.code, BridgeErrorCode::UnsupportedContractVersion);
        assert_eq!(error.request_id, "request-1");

        let invalid_id = BridgeRequest {
            bridge_contract_version: BRIDGE_CONTRACT_VERSION,
            request_id: "/tmp/private".to_owned(),
            payload: EmptyPayload::default(),
        };
        let error = validate_request(&invalid_id).expect_err("id should be rejected");
        assert_eq!(error.code, BridgeErrorCode::InvalidRequest);
        assert_eq!(error.request_id, "invalid-request");
        assert!(!serde_json::to_string(&error).unwrap().contains("/tmp"));

        let runtime = runtime_error("request-1", RuntimeError::StateUnavailable);
        assert_eq!(runtime.request_id, "request-1");
        assert_eq!(runtime.code, BridgeErrorCode::RuntimeNotReady);
    }

    #[test]
    fn status_event_is_versioned_and_redacted() {
        let event = bridge_status_event(&partial_snapshot());
        assert_eq!(event.bridge_contract_version, BRIDGE_CONTRACT_VERSION);
        assert_eq!(event.sequence, 1);
        assert_eq!(event.payload.runtime_phase, PublicRuntimePhase::Partial);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("streamId"));
        assert!(json.contains("runtimePhase"));
        assert!(!json.contains("not-configured"));
    }

    #[test]
    fn timestamps_use_utc_rfc3339_shape() {
        let timestamp = utc_timestamp();
        assert!(timestamp.ends_with('Z'));
        assert_eq!(timestamp.as_bytes().get(10), Some(&b'T'));
    }
}
