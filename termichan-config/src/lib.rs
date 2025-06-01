mod config;

// 公开导出配置相关的结构体和枚举，方便其他 crate 使用。
pub use config::{
    Config, ConfirmationMode, HistoryConfig, LlmConfig, NetworkConfig, OutputFormat, PromptConfig,
    SecurityConfig, UiConfig,
};

use confy;
use std::path::PathBuf;

/// 加载 `termichan` 配置，如果不存在则创建默认配置。
///
/// 使用 `confy` 来处理配置文件的加载。
/// 1. 如果提供了 `config_path_override`，则从该特定路径加载。
/// 2. 否则，使用 `confy::load` 从标准位置加载（例如 `~/.config/termichan/config.toml`）。
///
/// `confy` 会在文件不存在时自动尝试创建它，使用 `Config::default()` 并将其序列化为 TOML。
/// 它还会处理父目录的创建。
///
/// # Arguments
///
/// * `config_path_override` - 可选的配置文件路径，用于覆盖默认加载行为。
///
/// # Errors
///
/// 如果发生无法恢复的错误（例如，无法读取/写入文件权限问题，TOML 格式错误，无法创建目录等），
/// 则返回 `confy::ConfyError`。
///
/// # Returns
///
/// 成功时返回加载的 `Config` 实例。
pub fn load_or_create_config(config_path_override: Option<PathBuf>) -> Result<Config, confy::ConfyError> {
    let mut config: Config = match config_path_override {
        // 如果提供了覆盖路径，使用 confy::load_path。
        // confy::load_path 也会在文件不存在时尝试创建默认文件。
        Some(path) => confy::load_path(path),
        // 如果没有提供覆盖路径，使用 confy::load 让它处理标准路径和文件名。
        None => confy::load("termichan", None), // "termichan" 是应用名称，None 使用默认文件名 "config.toml"
    }?;

    // If api_key not exists, try load from env var
    if let None = config.llm.api_key {
        config.llm.api_key = std::env::var("OPENAI_API_KEY").ok();
        log::warn!("OPENAI_API_KEY isn't set in environment variable and config file.")
    }

    Ok(config)
}
