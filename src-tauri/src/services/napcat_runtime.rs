use std::{
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
    process::Command,
};

use rusqlite::Connection;
use serde::Serialize;
use serde_json::Value;

use crate::{
    error::AppError,
    repository::{accounts::AccountsRepository, settings::SettingsRepository},
};

pub const NAPCAT_VERSION: &str = "v4.17.53";
pub const NAPCAT_ASSET_NAME: &str = "NapCat.Shell.Windows.OneKey.zip";

const NAPCAT_RELEASE_BASE_URL: &str = "https://github.com/NapNeko/NapCatQQ/releases/download";
const NOTIFICATION_ENABLED_KEY: &str = "notification_enabled";
const NOTIFICATION_GROUP_ID_KEY: &str = "notification_group_id";
const NOTIFICATION_RUNTIME_VERSION_KEY: &str = "notification_runtime_version";
const NOTIFICATION_RUNTIME_INSTALL_DIR_KEY: &str = "notification_runtime_install_dir";
const NOTIFICATION_RUNTIME_PID_KEY: &str = "notification_runtime_pid";
const NOTIFICATION_ONEBOT_TOKEN_KEY: &str = "notification_onebot_token";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationPageStatusDto {
    pub env_status: String,
    pub is_enabled: bool,
    pub can_install_runtime: bool,
    pub runtime_version: String,
    pub install_dir: Option<String>,
    pub web_ui_url: Option<String>,
    pub one_bot_url: Option<String>,
    pub qq_number: Option<String>,
    pub group_id: String,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NapCatRuntimeState {
    pub is_supported_os: bool,
    pub runtime_installed: bool,
    pub runtime_running: bool,
    pub is_logged_in: bool,
    pub group_id: String,
}

#[derive(Debug, Clone)]
pub struct NapCatWebUiInfo {
    pub web_ui_url: String,
    pub one_bot_url: Option<String>,
    pub token: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NapCatWebUiInfoDto {
    pub web_ui_url: String,
    pub one_bot_url: Option<String>,
    pub token: Option<String>,
}

pub fn is_supported_windows_x64() -> bool {
    cfg!(target_os = "windows") && cfg!(target_arch = "x86_64")
}

pub fn install_dir() -> PathBuf {
    let base_dir = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir);
    base_dir
        .join("pubg-point-rankings")
        .join("napcat")
        .join(NAPCAT_VERSION)
}

#[derive(Debug, Clone)]
pub struct ExternalNapCatEnvironment {
    pub runtime_dir: PathBuf,
    pub webui_info: NapCatWebUiInfo,
}

pub fn resolve_runtime_dir(configured_install_dir: &str) -> PathBuf {
    if configured_install_dir.trim().is_empty() {
        install_dir()
    } else {
        PathBuf::from(configured_install_dir.trim())
    }
}

pub fn probe_external_environment(
    configured_install_dir: &str,
) -> Result<Option<ExternalNapCatEnvironment>, AppError> {
    let configured_dir = configured_install_dir.trim();
    if configured_dir.is_empty() {
        return Ok(None);
    }

    let runtime_dir = PathBuf::from(configured_dir);
    if !runtime_dir.exists() {
        return Ok(None);
    }

    let Some(webui_info) = discover_webui_url(&runtime_dir)? else {
        return Ok(None);
    };

    Ok(Some(ExternalNapCatEnvironment {
        runtime_dir,
        webui_info,
    }))
}

pub fn download_zip(destination: &Path) -> Result<(), AppError> {
    let download_url = format!(
        "{}/{}/{}",
        NAPCAT_RELEASE_BASE_URL, NAPCAT_VERSION, NAPCAT_ASSET_NAME
    );

    let response = ureq::get(&download_url).call().map_err(|error| {
        AppError::Message(format!("failed to download napcat runtime: {error}"))
    })?;

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut writer = BufWriter::new(File::create(destination)?);
    let mut reader = response.into_reader();
    io::copy(&mut reader, &mut writer)?;
    writer.flush()?;

    Ok(())
}

pub fn extract_zip(zip_path: &Path, target_dir: &Path) -> Result<(), AppError> {
    fs::create_dir_all(target_dir)?;

    let file = File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|error| AppError::Message(format!("failed to open zip archive: {error}")))?;

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| AppError::Message(format!("failed to read zip entry: {error}")))?;

        let Some(relative_path) = entry.enclosed_name().map(|path| path.to_owned()) else {
            continue;
        };

        let output_path = target_dir.join(relative_path);
        if entry.is_dir() {
            fs::create_dir_all(&output_path)?;
            continue;
        }

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut output_file = File::create(&output_path)?;
        io::copy(&mut entry, &mut output_file)?;
    }

    Ok(())
}

pub fn start_runtime(target_dir: &Path) -> Result<u32, AppError> {
    if !is_supported_windows_x64() {
        return Err(AppError::Message(
            "NapCat runtime is only supported on Windows x64".to_string(),
        ));
    }

    let bat_candidates = [
        target_dir.join("NapCat.Shell.Windows.OneKey.bat"),
        target_dir.join("start.bat"),
        target_dir.join("napcat.bat"),
    ];

    for candidate in bat_candidates {
        if candidate.is_file() {
            let child = Command::new("cmd")
                .arg("/C")
                .arg(candidate)
                .current_dir(target_dir)
                .spawn()?;

            return Ok(child.id());
        }
    }

    let exe_candidates = [
        target_dir.join("NapCat.Shell.exe"),
        target_dir.join("NapCat.exe"),
        target_dir.join("napcat.exe"),
    ];

    for candidate in exe_candidates {
        if candidate.is_file() {
            let child = Command::new(candidate).current_dir(target_dir).spawn()?;
            return Ok(child.id());
        }
    }

    Err(AppError::Message(
        "NapCat runtime start entry not found in install directory".to_string(),
    ))
}

pub fn stop_runtime(pid: u32) -> Result<(), AppError> {
    if !is_supported_windows_x64() {
        return Err(AppError::Message(
            "NapCat runtime is only supported on Windows x64".to_string(),
        ));
    }

    let output = Command::new("taskkill")
        .arg("/PID")
        .arg(pid.to_string())
        .arg("/T")
        .arg("/F")
        .output()
        .map_err(AppError::Io)?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("not found") || stderr.contains("没有找到") {
        return Ok(());
    }

    Err(AppError::Message(format!(
        "failed to stop napcat runtime process {pid}: {}",
        stderr.trim()
    )))
}

pub fn discover_webui_url(target_dir: &Path) -> Result<Option<NapCatWebUiInfo>, AppError> {
    let candidates = [
        target_dir.join("webui.json"),
        target_dir.join("config").join("webui.json"),
        target_dir
            .join("NapCat.Shell.Windows.OneKey")
            .join("config")
            .join("webui.json"),
    ];

    for path in candidates {
        if !path.is_file() {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let value: Value = serde_json::from_str(&content).map_err(|error| {
            AppError::Message(format!(
                "failed to parse webui config at {}: {error}",
                path.display()
            ))
        })?;

        let host = coerce_loopback_host(first_string(&value, &["host", "webuiHost", "webui_host"]));
        let webui_port = first_u64(&value, &["port", "webuiPort", "webui_port"]).unwrap_or(6099);
        let one_bot_port = first_u64(&value, &["onebotPort", "onebot_port"]);
        let token = first_string(&value, &["token", "accessToken", "access_token"]);

        let web_ui_url = first_string(&value, &["webUiUrl", "webuiUrl", "webui_url"])
            .unwrap_or_else(|| {
                if let Some(token_value) = token.as_ref().filter(|item| !item.is_empty()) {
                    format!("http://{host}:{webui_port}/?token={token_value}")
                } else {
                    format!("http://{host}:{webui_port}")
                }
            });

        let one_bot_url = first_string(&value, &["oneBotUrl", "onebotUrl", "onebot_url"])
            .or_else(|| one_bot_port.map(|port| format!("http://{host}:{port}")));

        return Ok(Some(NapCatWebUiInfo {
            web_ui_url,
            one_bot_url,
            token,
        }));
    }

    Ok(None)
}

pub fn query_login_info(one_bot_url: &str, token: &str) -> Result<Option<String>, AppError> {
    let endpoint = format!("{}/get_login_info", one_bot_url.trim_end_matches('/'));
    let mut request = ureq::get(&endpoint);

    if !token.is_empty() {
        request = request.set("Authorization", &format!("Bearer {token}"));
    }

    let response = request.call().map_err(|error| {
        AppError::Message(format!(
            "failed to query napcat login info from {endpoint}: {error}"
        ))
    })?;

    let body: Value = response.into_json().map_err(|error| {
        AppError::Message(format!("failed to parse napcat login response: {error}"))
    })?;

    let maybe_user_id = body
        .get("data")
        .and_then(|data| data.get("user_id"))
        .or_else(|| body.get("data").and_then(|data| data.get("uin")));

    let Some(user_id) = maybe_user_id else {
        return Ok(None);
    };

    match user_id {
        Value::String(text) => Ok((!text.trim().is_empty()).then_some(text.trim().to_string())),
        Value::Number(number) => Ok(Some(number.to_string())),
        _ => Ok(None),
    }
}

pub fn evaluate_env_status(state: &NapCatRuntimeState) -> &'static str {
    if !state.is_supported_os {
        return "unsupported_os";
    }

    if !state.runtime_installed {
        return "missing_runtime";
    }

    if !state.runtime_running {
        return "runtime_not_running";
    }

    if !state.is_logged_in {
        return "not_logged_in";
    }

    if state.group_id.trim().is_empty() {
        return "missing_group_id";
    }

    "ready"
}

pub fn get_notification_status(
    connection: &Connection,
) -> Result<NotificationPageStatusDto, AppError> {
    let account = AccountsRepository::new(connection).require_active()?;
    let settings = SettingsRepository::new(connection);

    let is_enabled = settings.get_account_bool(account.id, NOTIFICATION_ENABLED_KEY, false)?;
    let group_id = settings.get_account_string(account.id, NOTIFICATION_GROUP_ID_KEY, "")?;
    let configured_install_dir = settings.get_string(NOTIFICATION_RUNTIME_INSTALL_DIR_KEY, "")?;
    let runtime_version = settings.get_string(NOTIFICATION_RUNTIME_VERSION_KEY, NAPCAT_VERSION)?;

    let windows_managed = is_supported_windows_x64();
    let (is_supported_os, runtime_installed, runtime_dir, webui_info) = if windows_managed {
        let runtime_dir = resolve_runtime_dir(&configured_install_dir);
        let runtime_installed = runtime_dir.exists();
        let webui_info = if runtime_installed {
            discover_webui_url(&runtime_dir)?
        } else {
            None
        };

        (true, runtime_installed, Some(runtime_dir), webui_info)
    } else {
        let external_env = probe_external_environment(&configured_install_dir)?;
        match external_env {
            Some(environment) => (
                true,
                true,
                Some(environment.runtime_dir),
                Some(environment.webui_info),
            ),
            None => (false, false, None, None),
        }
    };

    let one_bot_url = webui_info
        .as_ref()
        .and_then(|info| info.one_bot_url.clone());
    let token = settings.get_account_string(account.id, NOTIFICATION_ONEBOT_TOKEN_KEY, "")?;
    let runtime_pid = settings.get_account_u32(account.id, NOTIFICATION_RUNTIME_PID_KEY)?;
    let pid_running = runtime_pid
        .map(is_process_running)
        .transpose()?
        .unwrap_or(false);

    let (qq_number, onebot_reachable) = if let Some(one_bot_url_value) = one_bot_url.as_deref() {
        let effective_token = webui_info
            .as_ref()
            .and_then(|info| info.token.as_ref())
            .filter(|item| !item.is_empty())
            .cloned()
            .unwrap_or(token);

        match query_login_info(one_bot_url_value, &effective_token) {
            Ok(number) => (number, true),
            Err(_) => (None, false),
        }
    } else {
        (None, false)
    };

    let runtime_running = pid_running || onebot_reachable;
    let env_state = NapCatRuntimeState {
        is_supported_os,
        runtime_installed,
        runtime_running,
        is_logged_in: qq_number.is_some(),
        group_id: group_id.clone(),
    };

    Ok(NotificationPageStatusDto {
        env_status: evaluate_env_status(&env_state).to_string(),
        is_enabled,
        can_install_runtime: windows_managed,
        runtime_version,
        install_dir: runtime_installed
            .then(|| {
                runtime_dir
                    .as_ref()
                    .map(|path| path.to_string_lossy().to_string())
            })
            .flatten(),
        web_ui_url: webui_info.as_ref().map(|info| info.web_ui_url.clone()),
        one_bot_url,
        qq_number,
        group_id,
        last_error: None,
    })
}

pub fn save_group_id_and_get_status(
    connection: &Connection,
    group_id: &str,
) -> Result<NotificationPageStatusDto, AppError> {
    let account = AccountsRepository::new(connection).require_active()?;
    SettingsRepository::new(connection).set_account(
        account.id,
        NOTIFICATION_GROUP_ID_KEY,
        group_id,
    )?;
    get_notification_status(connection)
}

pub fn install_runtime_and_get_status(
    connection: &Connection,
) -> Result<NotificationPageStatusDto, AppError> {
    if !is_supported_windows_x64() {
        return Err(AppError::Message(
            "NapCat runtime is only supported on Windows x64".to_string(),
        ));
    }

    let account = AccountsRepository::new(connection).require_active()?;
    let settings = SettingsRepository::new(connection);

    let target_dir = install_dir();
    let parent_dir = target_dir.parent().ok_or_else(|| {
        AppError::Message("failed to resolve napcat install directory parent".to_string())
    })?;
    fs::create_dir_all(parent_dir)?;

    let zip_path = parent_dir.join(NAPCAT_ASSET_NAME);
    download_zip(&zip_path)?;
    extract_zip(&zip_path, &target_dir)?;
    let _ = fs::remove_file(&zip_path);

    settings.set(NOTIFICATION_RUNTIME_VERSION_KEY, NAPCAT_VERSION)?;
    settings.set(
        NOTIFICATION_RUNTIME_INSTALL_DIR_KEY,
        &target_dir.to_string_lossy(),
    )?;
    settings.set_account(account.id, NOTIFICATION_RUNTIME_PID_KEY, "")?;

    get_notification_status(connection)
}

pub fn start_runtime_and_get_status(
    connection: &Connection,
) -> Result<NotificationPageStatusDto, AppError> {
    let account = AccountsRepository::new(connection).require_active()?;
    let settings = SettingsRepository::new(connection);
    let install_dir_value = settings.get_string(NOTIFICATION_RUNTIME_INSTALL_DIR_KEY, "")?;
    let target_dir = resolve_runtime_dir(&install_dir_value);

    if !target_dir.exists() {
        return Err(AppError::Message(
            "NapCat runtime is not installed. Install runtime first.".to_string(),
        ));
    }

    let pid = start_runtime(&target_dir)?;
    settings.set_account(account.id, NOTIFICATION_RUNTIME_PID_KEY, &pid.to_string())?;

    get_notification_status(connection)
}

pub fn stop_runtime_for_account(connection: &Connection) -> Result<(), AppError> {
    let account = AccountsRepository::new(connection).require_active()?;
    let settings = SettingsRepository::new(connection);
    let runtime_pid = settings.get_account_u32(account.id, NOTIFICATION_RUNTIME_PID_KEY)?;

    if let Some(pid) = runtime_pid {
        stop_runtime(pid)?;
    }

    settings.set_account(account.id, NOTIFICATION_RUNTIME_PID_KEY, "")?;
    Ok(())
}

pub fn restart_runtime_and_get_status(
    connection: &Connection,
) -> Result<NotificationPageStatusDto, AppError> {
    stop_runtime_for_account(connection)?;
    start_runtime_and_get_status(connection)
}

pub fn open_webui_info(connection: &Connection) -> Result<NapCatWebUiInfoDto, AppError> {
    let settings = SettingsRepository::new(connection);
    let configured_install_dir = settings.get_string(NOTIFICATION_RUNTIME_INSTALL_DIR_KEY, "")?;
    let runtime_dir = resolve_runtime_dir(&configured_install_dir);

    let webui_info = discover_webui_url(&runtime_dir)?
        .ok_or_else(|| AppError::Message("webui config not found".to_string()))?;

    Ok(NapCatWebUiInfoDto {
        web_ui_url: webui_info.web_ui_url,
        one_bot_url: webui_info.one_bot_url,
        token: webui_info.token,
    })
}

fn is_process_running(pid: u32) -> Result<bool, AppError> {
    if !cfg!(target_os = "windows") {
        return Ok(false);
    }

    let output = Command::new("tasklist")
        .arg("/FI")
        .arg(format!("PID eq {pid}"))
        .output()
        .map_err(AppError::Io)?;

    if !output.status.success() {
        return Ok(false);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.contains(&pid.to_string()))
}

fn first_string(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        value
            .get(*key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToOwned::to_owned)
    })
}

fn first_u64(value: &Value, keys: &[&str]) -> Option<u64> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_u64))
}

fn coerce_loopback_host(host: Option<String>) -> String {
    let normalized = host
        .as_deref()
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(|item| item.to_ascii_lowercase());

    match normalized.as_deref() {
        Some("127.0.0.1") | Some("localhost") | Some("::1") => {
            host.unwrap_or_else(|| "127.0.0.1".to_string())
        }
        _ => "127.0.0.1".to_string(),
    }
}

trait AccountSettingExt {
    fn get_account_bool(
        &self,
        account_id: i64,
        key: &str,
        default_value: bool,
    ) -> Result<bool, AppError>;
    fn get_account_u32(&self, account_id: i64, key: &str) -> Result<Option<u32>, AppError>;
}

impl<'a> AccountSettingExt for SettingsRepository<'a> {
    fn get_account_bool(
        &self,
        account_id: i64,
        key: &str,
        default_value: bool,
    ) -> Result<bool, AppError> {
        let fallback = if default_value { "1" } else { "0" };
        let value = self.get_account_string(account_id, key, fallback)?;
        let normalized = value.trim().to_ascii_lowercase();

        Ok(match normalized.as_str() {
            "1" | "true" | "yes" | "on" => true,
            "0" | "false" | "no" | "off" => false,
            _ => default_value,
        })
    }

    fn get_account_u32(&self, account_id: i64, key: &str) -> Result<Option<u32>, AppError> {
        let value = self.get_account_string(account_id, key, "")?;
        if value.trim().is_empty() {
            return Ok(None);
        }

        Ok(value.trim().parse::<u32>().ok())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use rusqlite::Connection;

    use crate::{db::migrations::bootstrap_database, repository::settings::SettingsRepository};

    use super::{
        coerce_loopback_host, evaluate_env_status, probe_external_environment, resolve_runtime_dir,
        save_group_id_and_get_status, NapCatRuntimeState, NOTIFICATION_RUNTIME_INSTALL_DIR_KEY,
    };

    fn make_temp_dir(prefix: &str) -> PathBuf {
        let unique = format!(
            "{}_{}",
            prefix,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after unix epoch")
                .as_nanos()
        );
        let path = std::env::temp_dir().join(unique);
        fs::create_dir_all(&path).expect("failed to create temp dir");
        path
    }

    #[test]
    fn reports_missing_runtime_before_login_and_group_checks() {
        let status = evaluate_env_status(&NapCatRuntimeState {
            is_supported_os: true,
            runtime_installed: false,
            runtime_running: false,
            is_logged_in: false,
            group_id: String::new(),
        });

        assert_eq!(status, "missing_runtime");
    }

    #[test]
    fn forces_non_loopback_host_to_localhost() {
        assert_eq!(
            coerce_loopback_host(Some("192.168.1.10".to_string())),
            "127.0.0.1"
        );
        assert_eq!(
            coerce_loopback_host(Some("localhost".to_string())),
            "localhost"
        );
    }

    #[test]
    fn resolves_runtime_dir_from_trimmed_configured_path() {
        let resolved = resolve_runtime_dir("   /tmp/napcat-runtime   ");
        assert_eq!(resolved, PathBuf::from("/tmp/napcat-runtime"));
    }

    #[test]
    fn probe_external_environment_returns_none_without_webui_config() {
        let temp_dir = make_temp_dir("napcat-probe-no-webui");
        let result = probe_external_environment(temp_dir.to_string_lossy().as_ref())
            .expect("probe should not fail");

        assert!(result.is_none());
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn probe_external_environment_detects_existing_webui_config() {
        let temp_dir = make_temp_dir("napcat-probe-with-webui");
        let webui_path = temp_dir.join("webui.json");
        fs::write(
            &webui_path,
            r#"{"host":"127.0.0.1","port":6099,"onebotPort":3001,"token":"abc"}"#,
        )
        .expect("failed to write test webui.json");

        let result = probe_external_environment(temp_dir.to_string_lossy().as_ref())
            .expect("probe should not fail");

        assert!(result.is_some());
        let info = result.expect("expected detected environment");
        assert_eq!(info.runtime_dir, temp_dir);
        assert_eq!(
            info.webui_info.web_ui_url,
            "http://127.0.0.1:6099/?token=abc"
        );
        assert_eq!(
            info.webui_info.one_bot_url,
            Some("http://127.0.0.1:3001".to_string())
        );
        let _ = fs::remove_dir_all(info.runtime_dir);
    }

    #[test]
    fn save_group_id_persists_account_setting_and_returns_updated_status() {
        let connection = Connection::open_in_memory().expect("open in-memory db");
        bootstrap_database(&connection).expect("bootstrap schema");

        connection
            .execute(
                "UPDATE accounts
                 SET account_name = 'steam', self_player_name = 'SelfPlayer', self_platform = 'steam', is_active = 1
                 WHERE id = 1",
                [],
            )
            .expect("ensure active account");

        let runtime_dir = make_temp_dir("napcat-save-group-id");
        fs::write(
            runtime_dir.join("webui.json"),
            r#"{"host":"127.0.0.1","port":6099,"token":"abc"}"#,
        )
        .expect("write webui config");

        SettingsRepository::new(&connection)
            .set(
                NOTIFICATION_RUNTIME_INSTALL_DIR_KEY,
                runtime_dir.to_string_lossy().as_ref(),
            )
            .expect("store runtime dir");

        let status = save_group_id_and_get_status(&connection, "123456")
            .expect("save group id should succeed");

        assert_eq!(status.group_id, "123456");
        assert_eq!(status.env_status, "runtime_not_running");

        let saved_group_id: String = connection
            .query_row(
                "SELECT value FROM account_settings WHERE account_id = 1 AND key = 'notification_group_id'",
                [],
                |row| row.get(0),
            )
            .expect("load saved group id");
        assert_eq!(saved_group_id, "123456");

        let _ = fs::remove_dir_all(runtime_dir);
    }
}
