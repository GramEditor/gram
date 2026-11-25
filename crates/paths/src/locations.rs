//! Paths to locations used by Gram.

use crate::{rel_path::RelPath, sanitized_path::SanitizedPath};
use std::{
    path::{Path, PathBuf},
    sync::{LazyLock, OnceLock},
};

// TODO: Comb the comments to ensure consistentcy with updates.

/// A default editorconfig file name to use when resolving project settings.
pub const EDITORCONFIG_NAME: &str = ".editorconfig";

/// The resolved data directory, combining custom override or platform defaults.
/// This is set once and cached for subsequent calls.
/// On macOS, this is `~/Library/Application Support/Gram`.
/// On Linux/FreeBSD, this is `$XDG_DATA_HOME/gram`.
/// On Windows, this is `%LOCALAPPDATA%\Gram`.
static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

/// The resolved config directory, combining custom override or platform defaults.
/// This is set once and cached for subsequent calls.
/// On macOS, this is `~/.config/gram`.
/// On Linux/FreeBSD, this is `$XDG_CONFIG_HOME/gram`.
/// On Windows, this is `%APPDATA%\Gram`.
/// Returns the relative path to the gram_server directory on the ssh host.
pub fn remote_server_dir_relative() -> &'static RelPath {
    static CACHED: LazyLock<&'static RelPath> = LazyLock::new(|| RelPath::unix(".gram_server").unwrap());

    *CACHED
}

/// Returns the relative path to the gram_wsl_server directory on the wsl host.
pub fn remote_wsl_server_dir_relative() -> &'static RelPath {
    static CACHED: LazyLock<&'static RelPath> = LazyLock::new(|| RelPath::unix(".gram_wsl_server").unwrap());

    *CACHED
}

/// Returns the path to the user's home directory.
pub fn home_dir() -> &'static Path {
    static HOME_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
        if cfg!(any(test, feature = "test-support")) {
            if cfg!(target_os = "macos") {
                PathBuf::from("/Users/gram")
            } else if cfg!(target_os = "windows") {
                PathBuf::from("C:\\Users\\gram")
            } else {
                PathBuf::from("/home/gram")
            }
        } else {
            dirs::home_dir().expect("failed to determine home directory")
        }
    });

    HOME_DIR.as_path()
}

/// Sets a custom directory for all user data, overriding the default data directory.
/// This function must be called before any other path operations that depend on the data directory.
/// The directory's path will be canonicalized to an absolute path by a blocking FS operation.
/// The directory will be created if it doesn't exist.
///
/// # Arguments
///
/// TODO: This dir is not used for logs on Mac OS or other platforms that provide dedicated
///       directories. Is it even worth providing a description of the function of each arg? It's obvious.
/// * `dir` - The path to use as the custom data directory. This will be used as the base
///   directory for all user data, including databases, extensions, and logs.
///
/// # Panics
///
/// Panics if:
/// * Called after the data directory has been initialized (e.g., via [`data_dir()`] or [`config_dir()`]).
/// * The directory's path cannot be canonicalized to an absolute path.
/// * The directory cannot be created.
pub fn set_data_dir(dir_path: impl AsRef<Path>) {
    DATA_DIR
        .set({
            let mut dir_path = dir_path.as_ref().to_owned();

            if dir_path.is_relative() && dir_path.exists() {
                let absolute_path = dir_path
                    .canonicalize()
                    .expect("failed to canonicalize custom data directory path");
                let sanitized_path = SanitizedPath::new(absolute_path.as_path());

                dir_path = PathBuf::from(sanitized_path);
            }

            std::fs::create_dir_all(dir_path.as_path())
                .expect("failed to create custom data or intermediary directory");

            dir_path
        })
        .expect("data directory path has already been initialized");
}

/// Returns the path to the data directory used by Gram.
pub fn data_dir() -> &'static Path {
    DATA_DIR
        .get_or_init(|| {
            let platform_data_dir = {
                if cfg!(any(target_os = "linux", target_os = "freebsd"))
                    && let Ok(flatpak_xdg_config) = std::env::var("FLATPAK_XDG_DATA_HOME")
                {
                    // TODO: Consider whether supporting this completely non-standard env var is useful
                    //       (it can just be set via XDG_DATA_HOME, so is pretty much pointless).
                    PathBuf::from(flatpak_xdg_config)
                } else {
                    dirs::data_local_dir().expect("failed to determine platform data directory")
                }
            };

            cfg_select! {
                any(target_os = "windows", target_os = "macos") => platform_data_dir.join("Gram"),
                _ => platform_data_dir.join("gram")
            }
        })
        .as_path()
}

/// Returns the path to the configuration directory used by Gram.
pub fn config_dir() -> &'static Path {
    static CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
        if let Some(custom_dir) = DATA_DIR.get() {
            // Use the custom-set user data dir as the antecedent config directory.
            return custom_dir.join("config");
        }

        let platform_config_dir = {
            if cfg!(any(target_os = "linux", target_os = "freebsd"))
                && let Ok(flatpak_xdg_config) = std::env::var("FLATPAK_XDG_CONFIG_HOME")
            {
                // TODO: Consider whether supporting this completely non-standard env var is useful
                //       (it can just be set via XDG_CONFIG_HOME, so is pretty much pointless).
                PathBuf::from(flatpak_xdg_config)
            } else {
                // TODO: This should probably be `dirs::config_local_dir`.
                //       To exemplify, using `config_dir` instead of `config_local_dir` causes the application
                //       config directory to be in `%appdata%/Roaming`, while the application data directory
                //       is in `%appdata%/LocalLow`.
                dirs::config_dir().expect("failed to determine platform config directory")
            }
        };

        cfg_select! {
            any(target_os = "windows", target_os = "macos") => platform_config_dir.join("Gram"),
            _ => platform_config_dir.join("gram")
        }
    });

    CONFIG_DIR.as_path()
}

/// Returns the path to the temp directory used by Gram.
pub fn temp_dir() -> &'static Path {
    static TEMP_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
        let platform_cache_dir = {
            if cfg!(any(target_os = "linux", target_os = "freebsd"))
                && let Ok(flatpak_xdg_config) = std::env::var("FLATPAK_XDG_CACHE_HOME")
            {
                // TODO: Consider whether supporting this completely non-standard env var is useful
                //       (it can just be set via XDG_CACHE_HOME, so is pretty much pointless).
                PathBuf::from(flatpak_xdg_config)
            } else {
                // For all other platforms, simply use the default.
                dirs::cache_dir().expect("failed to determine platform cache directory")
            }
        };

        cfg_select! {
            any(target_os = "windows", target_os = "macos") => platform_cache_dir.join("Gram"),
            _ => platform_cache_dir.join("gram")
        }
    });

    TEMP_DIR.as_path()
}

/// Returns the path to the logs directory.
pub fn logs_dir() -> &'static Path {
    static LOGS_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
        cfg_select! {
            target_os = "macos" => home_dir().join("Library/Logs/Gram"),
            _ => data_dir().join("logs")
        }
    });

    LOGS_DIR.as_path()
}

/// Returns the path to the hang traces directory.
pub fn hang_traces_dir() -> &'static Path {
    static HANG_TRACES_DIR: LazyLock<PathBuf> = LazyLock::new(|| logs_dir().join("hang_traces"));

    HANG_TRACES_DIR.as_path()
}

/// Returns the path to the Gram server directory on this SSH host.
pub fn remote_server_state_dir() -> &'static Path {
    static REMOTE_SERVER_STATE_DIR: LazyLock<PathBuf> = LazyLock::new(|| data_dir().join("server_state"));

    REMOTE_SERVER_STATE_DIR.as_path()
}

/// Returns the path to the `Gram.log` file.
pub fn log_file() -> &'static Path {
    static LOG_FILE: LazyLock<PathBuf> = LazyLock::new(|| logs_dir().join("Gram.log"));

    LOG_FILE.as_path()
}

/// Returns the path to the `Gram.log.old` file.
pub fn old_log_file() -> &'static Path {
    static OLD_LOG_FILE: LazyLock<PathBuf> = LazyLock::new(|| log_file().with_added_extension("old"));

    OLD_LOG_FILE.as_path()
}

/// Returns the path to the database directory.
pub fn database_dir() -> &'static Path {
    static DATABASE_DIR: LazyLock<PathBuf> = LazyLock::new(|| data_dir().join("db"));

    DATABASE_DIR.as_path()
}

/// Returns the path to the crashes directory, if it exists for the current platform.
pub fn crashes_dir() -> Option<&'static Path> {
    static CRASHES_DIR: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
        cfg_select! {
            target_os = "macos" => Some(home_dir().join("Library/Logs/DiagnosticReports")),
            _ => None
        }
    });

    CRASHES_DIR.as_ref().map(PathBuf::as_path)
}

/// Returns the path to the `settings.jsonc` file.
pub fn settings_file() -> &'static Path {
    static SETTINGS_FILE: LazyLock<PathBuf> = LazyLock::new(|| config_dir().join("settings.jsonc"));

    SETTINGS_FILE.as_path()
}

/// Returns the path to the global settings file.
pub fn global_settings_file() -> &'static Path {
    static GLOBAL_SETTINGS_FILE: LazyLock<PathBuf> = LazyLock::new(|| config_dir().join("global_settings.jsonc"));

    GLOBAL_SETTINGS_FILE.as_path()
}

/// Returns the path to the `settings_backup.json` file.
pub fn settings_backup_file() -> &'static Path {
    static SETTINGS_BACKUP_FILE: LazyLock<PathBuf> = LazyLock::new(|| config_dir().join("settings.backup.jsonc"));

    SETTINGS_BACKUP_FILE.as_path()
}

/// Returns the path to the `keymap.json` file.
pub fn keymap_file() -> &'static Path {
    static KEYMAP_FILE: LazyLock<PathBuf> = LazyLock::new(|| config_dir().join("keymap.jsonc"));

    KEYMAP_FILE.as_path()
}

/// Returns the path to the `keymap_backup.json` file.
pub fn keymap_backup_file() -> &'static Path {
    static KEYMAP_BACKUP_FILE: LazyLock<PathBuf> = LazyLock::new(|| config_dir().join("keymap.backup.jsonc"));

    KEYMAP_BACKUP_FILE.as_path()
}

/// Returns the path to the `tasks.jsonc` file.
pub fn tasks_file() -> &'static Path {
    static TASKS_FILE: LazyLock<PathBuf> = LazyLock::new(|| config_dir().join("tasks.jsonc"));

    TASKS_FILE.as_path()
}

/// Returns the path to the `debug.jsonc` file.
pub fn debug_scenarios_file() -> &'static Path {
    static DEBUG_SCENARIOS_FILE: LazyLock<PathBuf> = LazyLock::new(|| config_dir().join("debug.jsonc"));

    DEBUG_SCENARIOS_FILE.as_path()
}

/// Returns the path to the user extensions directory.
///
/// This is where user-installed extensions are stored.
pub fn user_extensions_dir() -> &'static Path {
    static EXTENSIONS_DIR: LazyLock<PathBuf> = LazyLock::new(|| data_dir().join("extensions"));

    EXTENSIONS_DIR.as_path()
}

/// Returns the path to the (optional) system extensions directory.
///
/// This is where system-installed extensions are stored.
pub fn system_extensions_dir() -> Option<&'static Path> {
    static SYSTEM_EXTENSIONS_DIR: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
        match std::env::var("GRAM_SYSTEM_EXTENSIONS_DIR") {
            Ok(dir) => Some(PathBuf::from(dir)),

            Err(_) => {
                // While the error state may be `env::VarError::NotUnicode(_)`, it is not handled
                // separately at the moment. This could potentially be changed, and would be a small improvement.

                option_env!("GRAM_SYSTEM_EXTENSIONS_DIR").map(PathBuf::from)
            }
        }
    });

    SYSTEM_EXTENSIONS_DIR.as_ref().map(PathBuf::as_path)
}

/// This is where installed extensions are stored on a remote.
pub fn remote_extensions_dir() -> &'static Path {
    static EXTENSIONS_DIR: LazyLock<PathBuf> = LazyLock::new(|| data_dir().join("remote_extensions"));

    EXTENSIONS_DIR.as_path()
}

/// This is where installed extensions are uploaded on a remote.
pub fn remote_extensions_uploads_dir() -> &'static Path {
    static REMOTE_EXTENSIONS_UPLOADS_DIR: LazyLock<PathBuf> = LazyLock::new(|| remote_extensions_dir().join("uploads"));

    REMOTE_EXTENSIONS_UPLOADS_DIR.as_path()
}

/// Returns the path to the themes directory.
///
/// This is where themes that are not provided by extensions are stored.
pub fn themes_dir() -> &'static Path {
    static THEMES_DIR: LazyLock<PathBuf> = LazyLock::new(|| config_dir().join("themes"));

    THEMES_DIR.as_path()
}

/// Returns the path to the snippets directory.
pub fn snippets_dir() -> &'static Path {
    static SNIPPETS_DIR: LazyLock<PathBuf> = LazyLock::new(|| config_dir().join("snippets"));

    SNIPPETS_DIR.as_path()
}

/// Returns the path to the languages directory.
///
/// This is where language servers for built-in languages are downloaded.
pub fn languages_dir() -> &'static Path {
    static LANGUAGES_DIR: LazyLock<PathBuf> = LazyLock::new(|| data_dir().join("languages"));

    LANGUAGES_DIR.as_path()
}

/// Returns the path to the debug adapters directory
///
/// This is where debug adapters are downloaded to for built-in DAPs.
pub fn debug_adapters_dir() -> &'static Path {
    static DEBUG_ADAPTERS_DIR: LazyLock<PathBuf> = LazyLock::new(|| data_dir().join("debug_adapters"));

    DEBUG_ADAPTERS_DIR.as_path()
}

/// Returns the path to the default Prettier directory.
pub fn default_prettier_dir() -> &'static Path {
    static DEFAULT_PRETTIER_DIR: LazyLock<PathBuf> = LazyLock::new(|| data_dir().join("prettier"));

    DEFAULT_PRETTIER_DIR.as_path()
}

/// Returns the relative path to a `.gram` folder within a project.
pub fn local_settings_folder_name() -> &'static str {
    ".gram"
}

/// Returns the relative path to a `.vscode` folder within a project.
pub fn local_vscode_folder_name() -> &'static str {
    ".vscode"
}

/// Returns the relative path to a `.vscodium` folder within a project.
pub fn local_vscodium_folder_name() -> &'static str {
    ".vscodium"
}

pub fn debug_task_file_name() -> &'static str {
    "debug.jsonc"
}

pub fn task_file_name() -> &'static str {
    "tasks.jsonc"
}

/// Returns the relative path to a `settings.jsonc` file within a project.
pub fn local_settings_file_relative_path() -> &'static RelPath {
    // TODO: use `local_settings_folder_name()` instead of hard-coding.

    static LOCAL_SETTINGS_FILE: LazyLock<&'static RelPath> =
        LazyLock::new(|| RelPath::unix(".gram/settings.jsonc").unwrap());

    *LOCAL_SETTINGS_FILE
}

/// Returns the relative path to a `tasks.jsonc` file within a project.
pub fn local_tasks_file_relative_path() -> &'static RelPath {
    // TODO: use `local_settings_folder_name()` + `task_file_name()` instead of hard-coding.

    static LOCAL_TASKS_FILE: LazyLock<&'static RelPath> = LazyLock::new(|| RelPath::unix(".gram/tasks.jsonc").unwrap());

    *LOCAL_TASKS_FILE
}

/// Returns the relative path to a `.vscode/tasks.json` file within a project.
pub fn local_vscode_tasks_file_relative_path() -> &'static RelPath {
    // TODO: use `local_vscode_folder_name()` instead of hard-coding.

    static LOCAL_VSCODE_TASKS_FILE: LazyLock<&'static RelPath> =
        LazyLock::new(|| RelPath::unix(".vscode/tasks.json").unwrap());

    *LOCAL_VSCODE_TASKS_FILE
}

/// Returns the relative path to a `.vscode/tasks.json` file within a project.
pub fn local_vscodium_tasks_file_relative_path() -> &'static RelPath {
    // TODO: use `local_vscodium_folder_name()` instead of hard-coding.

    static LOCAL_VSCODIUM_TASKS_FILE: LazyLock<&'static RelPath> =
        LazyLock::new(|| RelPath::unix(".vscodium/tasks.json").unwrap());

    *LOCAL_VSCODIUM_TASKS_FILE
}

/// Returns the relative path to a `debug.json` file within a project.
/// .gram/debug.json
pub fn local_debug_file_relative_path() -> &'static RelPath {
    // TODO: use `local_settings_folder_name()` + `debug_task_file_name()` instead of hard-coding.

    static LOCAL_DEBUG_FILE: LazyLock<&'static RelPath> = LazyLock::new(|| RelPath::unix(".gram/debug.jsonc").unwrap());

    *LOCAL_DEBUG_FILE
}

/// Returns the relative path to a `.vscode/launch.json` file within a project.
pub fn local_vscode_launch_file_relative_path() -> &'static RelPath {
    static LOCAL_VSCODE_LAUNCH_FILE: LazyLock<&'static RelPath> =
        LazyLock::new(|| RelPath::unix(".vscode/launch.json").unwrap());

    *LOCAL_VSCODE_LAUNCH_FILE
}

/// Returns the relative path to a `.vscode/launch.json` file within a project.
pub fn local_vscodium_launch_file_relative_path() -> &'static RelPath {
    static LOCAL_VSCODIUM_LAUNCH_FILE: LazyLock<&'static RelPath> =
        LazyLock::new(|| RelPath::unix(".vscodium/launch.json").unwrap());

    *LOCAL_VSCODIUM_LAUNCH_FILE
}

pub fn user_ssh_config_file() -> &'static Path {
    static USER_SSH_CONFIG_FILE: LazyLock<PathBuf> = LazyLock::new(|| home_dir().join(".ssh/config"));

    USER_SSH_CONFIG_FILE.as_path()
}

pub fn global_ssh_config_file() -> Option<&'static Path> {
    static GLOBAL_SSH_CONFIG_FILE: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
        cfg_select! {
            target_os = "windows" => None,
            _ => Some(PathBuf::from("/etc/ssh/ssh_config"))
        }
    });

    GLOBAL_SSH_CONFIG_FILE.as_ref().map(PathBuf::as_path)
}

fn add_vscode_user_data_paths(paths: &mut Vec<PathBuf>, product_name: &str) {
    cfg_select! {
        target_os = "macos" => paths.push(home_dir().join("Library/Application Support").join(product_name)),

        target_os = "windows" => {
            if let Some(data_local_dir) = dirs::data_local_dir() {
                paths.push(data_local_dir.join(product_name));
            }

            if let Some(data_dir) = dirs::data_dir() {
                paths.push(data_dir.join(product_name));
            }
        }

        _ => paths.push(dirs::config_dir().unwrap_or(home_dir().join(".config")).join(product_name)),
    }
}

fn vscode_user_data_paths() -> impl Iterator<Item = &'static Path> + Clone {
    static VSCODE_USER_DATA_PATHS: LazyLock<Vec<PathBuf>> = LazyLock::new(|| {
        // https://github.com/microsoft/vscode/blob/23e7148cdb6d8a27f0109ff77e5b1e019f8da051/src/vs/platform/environment/node/userDataPath.ts#L45
        const VSCODE_PRODUCT_NAMES: &[&str] = &[
            "Code",
            "Code - OSS",
            "VSCodium",
            "Code Dev",
            "Code - OSS Dev",
            "code-oss-dev",
        ];

        let mut paths = Vec::new();

        if let Ok(portable_path) = std::env::var("VSCODE_PORTABLE") {
            paths.push(Path::new(&portable_path).join("user-data"));
        }

        if let Ok(vscode_appdata) = std::env::var("VSCODE_APPDATA") {
            for product_name in VSCODE_PRODUCT_NAMES {
                paths.push(Path::new(&vscode_appdata).join(product_name));
            }
        }

        for product_name in VSCODE_PRODUCT_NAMES {
            add_vscode_user_data_paths(&mut paths, product_name);
        }

        paths
    });

    VSCODE_USER_DATA_PATHS.iter().map(PathBuf::as_path)
}

fn cursor_user_data_paths() -> impl Iterator<Item = &'static Path> + Clone {
    static CURSOR_USER_DATA_PATHS: LazyLock<Vec<PathBuf>> = LazyLock::new(|| {
        let mut paths = Vec::new();
        add_vscode_user_data_paths(&mut paths, "Cursor");
        paths
    });

    CURSOR_USER_DATA_PATHS.iter().map(PathBuf::as_path)
}

/// Returns candidate paths for the vscode user settings file
pub fn vscode_settings_file_paths() -> impl Iterator<Item = &'static Path> + Clone {
    static VSCODE_SETTINGS_FILE_PATHS: LazyLock<Vec<PathBuf>> = LazyLock::new(|| {
        vscode_user_data_paths()
            .map(|p| p.join("User/settings.json"))
            .collect::<Vec<PathBuf>>()
    });

    VSCODE_SETTINGS_FILE_PATHS.iter().map(PathBuf::as_path)
}

/// Returns candidate paths for the cursor user settings file
pub fn cursor_settings_file_paths() -> impl Iterator<Item = &'static Path> + Clone {
    static CURSOR_SETTINGS_FILE_PATHS: LazyLock<Vec<PathBuf>> = LazyLock::new(|| {
        cursor_user_data_paths()
            .map(|p| p.join("User/settings.json"))
            .collect::<Vec<PathBuf>>()
    });

    CURSOR_SETTINGS_FILE_PATHS.iter().map(PathBuf::as_path)
}

#[cfg(any(test, feature = "test-support"))]
#[expect(
    clippy::unnecessary_wraps,
    reason = "The other version of this function can actually return None."
)]
pub fn global_gitignore_path() -> Option<PathBuf> {
    Some(home_dir().join(".config").join("git").join("ignore"))
}

#[cfg(not(any(test, feature = "test-support")))]
pub fn global_gitignore_path() -> Option<PathBuf> {
    static GLOBAL_GITIGNORE_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();
    GLOBAL_GITIGNORE_PATH
        .get_or_init(::ignore::gitignore::gitconfig_excludes_path)
        .clone()
}
