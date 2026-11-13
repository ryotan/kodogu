use serde::Serialize;
use serde_json::Value;
use suzunari_error::StackError;
use tauri::State;

use super::store::{AppConfigStore, ConfigError};
use super::translators::{ConfigResponse, translate_request};

#[derive(Clone, Copy, Debug, Serialize)]
enum CommandErrorCode {
    #[serde(rename = "configuration_rejected")]
    Rejected,
    #[serde(rename = "configuration_unavailable")]
    Unavailable,
    #[serde(rename = "configuration_unrepresentable")]
    Unrepresentable,
}

#[derive(Debug, Serialize)]
pub(crate) struct CommandError {
    code: CommandErrorCode,
    message: &'static str,
}

impl CommandError {
    fn new(code: CommandErrorCode, cause: &impl StackError) -> Self {
        tracing::warn!(
            ?code,
            cause = cause.type_name(),
            source = cause.stack_source().map(StackError::type_name),
            location = %cause.location(),
            "Configuration command failed"
        );
        let message = match code {
            CommandErrorCode::Rejected => "Configuration request was rejected",
            CommandErrorCode::Unavailable => "Configuration storage is unavailable",
            CommandErrorCode::Unrepresentable => "Configuration cannot be represented by the GUI",
        };
        Self { code, message }
    }
}

impl From<ConfigError> for CommandError {
    fn from(cause: ConfigError) -> Self {
        let code = match &cause {
            ConfigError::Storage { .. } if cause.is_known_rejection() => CommandErrorCode::Rejected,
            ConfigError::CodecFailure { .. } => CommandErrorCode::Rejected,
            ConfigError::CachePoisoned { .. }
            | ConfigError::InvalidDefault { .. }
            | ConfigError::Unavailable { .. }
            | ConfigError::UnsupportedVersion { .. }
            | ConfigError::Transaction { .. }
            | ConfigError::Table { .. }
            | ConfigError::Storage { .. }
            | ConfigError::Commit { .. } => CommandErrorCode::Unavailable,
        };
        Self::new(code, &cause)
    }
}

#[tauri::command]
pub(crate) async fn get_config(
    state: State<'_, AppConfigStore>,
) -> Result<ConfigResponse, CommandError> {
    ConfigResponse::try_from(state.get()?)
        .map_err(|cause| CommandError::new(CommandErrorCode::Unrepresentable, &cause))
}

#[tauri::command]
pub(crate) async fn set_config(
    state: State<'_, AppConfigStore>,
    config: Value,
) -> Result<(), CommandError> {
    state.ensure_available()?;
    let update = translate_request(config)
        .map_err(|cause| CommandError::new(CommandErrorCode::Rejected, &cause))?;
    state.set(update)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::domain::AppConfig;
    use redb::{Database, backends::InMemoryBackend};
    use serde_json::json;
    use tauri::test::{MockRuntime, get_ipc_response, mock_builder, mock_context, noop_assets};

    fn window() -> tauri::WebviewWindow<MockRuntime> {
        let db = Database::builder().create_with_backend(InMemoryBackend::new()).unwrap();
        let app = mock_builder()
            .manage(AppConfigStore::new(db).unwrap())
            .invoke_handler(tauri::generate_handler![get_config, set_config])
            .build(mock_context(noop_assets()))
            .unwrap();
        tauri::WebviewWindowBuilder::new(&app, "popover", Default::default())
            .build()
            .unwrap()
    }

    fn invoke(
        window: &tauri::WebviewWindow<MockRuntime>,
        command: &str,
        body: Value,
    ) -> Result<Value, Value> {
        let origin = if cfg!(windows) { "http://tauri.localhost" } else { "tauri://localhost" };
        get_ipc_response(
            window,
            tauri::webview::InvokeRequest {
                cmd: command.into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: origin.parse().unwrap(),
                body: tauri::ipc::InvokeBody::Json(body),
                headers: Default::default(),
                invoke_key: tauri::test::INVOKE_KEY.to_owned(),
            },
        )
        .map(|response| response.deserialize().unwrap())
    }

    #[test]
    fn a_complete_replacement_is_returned_by_the_next_read() {
        let window = window();
        let defaults = serde_json::to_value(AppConfig::default()).unwrap();
        assert_eq!(invoke(&window, "get_config", json!({})), Ok(defaults.clone()));

        let mut replacement = defaults;
        replacement["popover"]["behavior"]["toggleDebounceMs"] = json!(0);
        assert_eq!(
            invoke(&window, "set_config", json!({ "config": &replacement })),
            Ok(Value::Null)
        );
        assert_eq!(invoke(&window, "get_config", json!({})), Ok(replacement));
    }

    #[test]
    fn an_incomplete_replacement_keeps_the_confirmed_value_and_returns_a_safe_error() {
        let window = window();
        let before = invoke(&window, "get_config", json!({})).unwrap();
        assert_eq!(
            invoke(&window, "set_config", json!({ "config": { "privateField": "secret" } })),
            Err(
                json!({ "code": "configuration_rejected", "message": "Configuration request was rejected" })
            )
        );
        assert_eq!(invoke(&window, "get_config", json!({})), Ok(before));
    }
}
