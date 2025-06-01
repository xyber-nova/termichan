use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// `termichan` 的主配置结构体。
///
/// 这个结构体包含了运行 `termichan` 所需的所有配置选项。
/// 它可以通过 TOML 文件进行配置，并使用 `serde` 进行序列化和反序列化。
/// 配置项被组织到不同的模块中，以提高可读性和可维护性。
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)] // 为所有未在配置文件中指定的字段提供默认值
pub struct Config {
    /// LLM (大型语言模型) 相关配置。
    pub llm: LlmConfig,
    /// 安全相关配置，特别是命令执行前的确认。
    pub security: SecurityConfig,
    /// 命令历史记录相关配置。
    pub history: HistoryConfig,
    /// 与 LLM 交互时使用的提示词配置。
    pub prompt: PromptConfig,
    /// 用户界面和输出格式化相关配置。
    pub ui: UiConfig,
    /// 网络连接相关配置，例如代理设置。
    pub network: NetworkConfig,
}

/// 为 `Config` 提供默认值。
///
/// 当用户没有提供配置文件或配置文件缺少某些部分时，将使用这些默认值。
impl Default for Config {
    fn default() -> Self {
        Self {
            llm: LlmConfig::default(),
            security: SecurityConfig::default(),
            history: HistoryConfig::default(),
            prompt: PromptConfig::default(),
            ui: UiConfig::default(),
            network: NetworkConfig::default(),
        }
    }
}

/// LLM (大型语言模型) 相关配置。
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct LlmConfig {
    /// 使用的 LLM 服务提供商。
    ///
    /// 这决定了 API 的调用方式和可能支持的模型。
    /// 例如: "openai", "google", "anthropic", "ollama", "custom" 等。
    pub provider: String,

    /// LLM API 密钥。
    ///
    /// **安全警告**: 强烈建议不要将密钥直接写入配置文件。
    /// 推荐使用环境变量 (例如 `OPENAI_API_KEY`) 或专门的密钥管理工具。
    /// 如果此字段为 `None`，应用程序应尝试从环境变量加载密钥。
    pub api_key: Option<String>,

    /// LLM API 的基础 URL (可选)。
    ///
    /// 对于 OpenAI 兼容的 API (如 `ollama` 或本地模型服务) 或需要代理访问时很有用。
    /// 如果为 `None`，则使用所选提供商的默认 API 端点。
    pub base_url: Option<String>,

    /// 要使用的具体模型名称。
    ///
    /// 确保所选模型与提供商和 API 密钥兼容。
    /// 例如: "gpt-4o", "gpt-3.5-turbo", "gemini-1.5-pro", "claude-3-opus-20240229"。
    pub model: String,

    /// 控制生成文本随机性的参数 (例如 OpenAI 的 temperature)。
    ///
    /// 值越高 (例如 0.8)，输出越具创造性和随机性；值越低 (例如 0.2)，输出越确定和集中。
    /// 典型范围是 0.0 到 2.0。设置为 0.0 可能等同于贪心解码。
    pub temperature: f32,

    /// 控制生成文本多样性的参数 (例如 OpenAI 的 top_p)。
    ///
    /// 核心采样：模型仅考虑概率总和达到 `top_p` 的最小词汇集。
    /// 通常建议只修改 `temperature` 或 `top_p` 中的一个。
    /// 典型范围是 0.0 到 1.0。
    pub top_p: Option<f32>,

    /// 生成响应的最大 token 数量限制。
    ///
    /// 这有助于控制 API 成本和响应时间。需要考虑输入 token 和输出 token 的总和限制。
    pub max_tokens: Option<u32>,

    /// API 请求的超时时间 (以秒为单位)。
    ///
    /// 防止应用程序因网络问题或 LLM 服务响应缓慢而无限期挂起。
    pub timeout_secs: u64,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(), // 默认使用 OpenAI
            api_key: None, // 强烈建议通过环境变量设置
            base_url: None,
            model: "gpt-4o".to_string(), // 默认使用最新的 OpenAI 模型之一
            temperature: 0.7,
            top_p: None, // 通常不与 temperature 同时设置
            max_tokens: Some(1500), // 为命令生成和解释提供足够空间
            timeout_secs: 60, // 1 分钟超时
        }
    }
}

/// 安全相关配置。
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct SecurityConfig {
    /// 命令执行前的确认策略。
    ///
    /// 控制 `termichan` 在执行 LLM 生成的命令之前是否需要用户确认。
    pub confirmation_mode: ConfirmationMode,

    /// (可选) 需要特别确认的“危险”命令列表。
    ///
    /// 仅在 `confirmation_mode` 设置为 `Dangerous` 时生效。
    /// 列表中的字符串将用于匹配生成命令的开头部分。
    /// 如果命令以列表中的任何一个字符串开头，将强制要求用户确认。
    /// **注意**: 这个列表可能不全面，依赖于简单的字符串匹配。
    pub dangerous_commands: Vec<String>,
}

/// 定义命令执行确认的不同模式。
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ConfirmationMode {
    /// `Always`: 在执行任何生成的命令之前总是需要用户确认。这是最安全的选择。
    Always,
    /// `Never`: 从不要求用户确认，直接执行生成的命令。
    /// **极度危险**: 仅在完全信任 LLM 输出且了解潜在风险时使用。
    Never,
    /// `Dangerous`: 仅对被识别为“危险”的命令要求确认（基于 `dangerous_commands` 列表）。
    /// 其他命令将不经确认直接执行。
    Dangerous,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            confirmation_mode: ConfirmationMode::Always, // 默认总是需要确认，安全第一
            dangerous_commands: vec![
                "rm ".to_string(),      // 删除文件/目录
                "sudo ".to_string(),    // 以超级用户权限执行
                "mv ".to_string(),      // 移动/重命名，可能覆盖文件
                "dd ".to_string(),      // 低级复制，可能破坏磁盘
                "mkfs".to_string(),     // 创建文件系统，格式化分区
                "shutdown ".to_string(), // 关闭系统
                "reboot".to_string(),   // 重启系统
                ":(){:|:&};:".to_string(), // Bash Fork Bomb
                "> /dev/sda".to_string(), // 覆盖块设备
                "chmod -R 000".to_string(), // 移除所有权限
                "chown -R nobody".to_string(), // 更改所有权
            ],
        }
    }
}

/// 历史记录相关配置。
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct HistoryConfig {
    /// 是否启用命令历史记录功能。
    ///
    /// 如果启用，`termichan` 将保存用户查询和生成的命令。
    pub enabled: bool,

    /// 历史记录文件的存储路径。
    ///
    /// 可以是绝对路径或相对路径。相对路径通常相对于用户配置目录。
    /// 例如: "~/.config/termichan/history.log" 或 "termichan_history.log"。
    pub file_path: PathBuf,

    /// 保存在历史记录文件中的最大条目数。
    ///
    /// 当历史记录达到此大小时，最旧的条目将被删除。
    pub max_entries: usize,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        // 尝试获取平台特定的用户配置目录
        let default_path = dirs::config_dir()
            .map(|p| p.join("termichan").join("history.log")) // 例如 ~/.config/termichan/history.log
            .or_else(|| dirs::home_dir().map(|p| p.join(".termichan_history.log"))) // 备选方案: ~/.termichan_history.log
            .unwrap_or_else(|| PathBuf::from("termichan_history.log")); // 最后备选: 当前目录

        Self {
            enabled: true, // 默认启用历史记录
            file_path: default_path,
            max_entries: 1000, // 保留最近 1000 条记录
        }
    }
}

/// 提示词相关配置。
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct PromptConfig {
    /// 自定义系统提示词 (System Prompt)。
    ///
    /// 这个提示词在每次与 LLM 交互开始时发送，用于设定 LLM 的角色、行为、上下文和输出要求。
    /// 可以包含占位符，这些占位符将在运行时被替换：
    /// - `{shell}`: 当前运行的 shell 类型 (例如 "bash", "zsh", "fish", "powershell")。
    /// - `{os}`: 当前操作系统 (例如 "linux", "macos", "windows")。
    /// - `{pwd}`: 当前工作目录。
    pub system_prompt: String,

    /// 用户输入的模板。
    ///
    /// 定义如何将用户的原始输入包装后发送给 LLM。
    /// 可以包含占位符：
    /// - `{user_input}`: 用户输入的原始文本。
    pub user_prompt_template: String,
}

impl Default for PromptConfig {
    fn default() -> Self {
        let system_prompt = r#"You are termichan, an expert AI assistant specialized in generating accurate and safe terminal commands based on user requests.
Your goal is to provide a single, executable command line that achieves the user's goal.

Current Environment:
- Operating System: {os}
- Shell: {shell}
- Working Directory: {pwd}

Guidelines:
1.  **Clarity:** Provide only the command itself, without any introductory phrases like "Here's the command:" or "You can use:".
2.  **Safety:** Prioritize safety. Avoid destructive commands unless explicitly requested and clearly necessary. If a potentially dangerous command is needed, add a brief comment `# Be careful: <reason>` after the command.
3.  **Conciseness:** Generate the most concise command that fulfills the request.
4.  **Placeholders:** If specific information is missing (e.g., filename, hostname), use clear placeholders like `<filename>` or `<hostname>` and indicate that the user needs to replace them.
5.  **Explanation (Optional):** If the command is complex or non-obvious, you MAY add a short explanation starting with `# Explanation:` on a new line after the command. Keep it brief.
6.  **No Markdown:** Do not use markdown formatting (like ```bash ... ```). Output only the raw command and optional comments/explanations.

Example Request: Find all files modified in the last 2 days
Example Response:
find . -type f -mtime -2

Example Request: Delete the logs folder
Example Response:
rm -rf ./logs # Be careful: This will permanently delete the folder and its contents.

Example Request: Show disk usage for the current directory
Example Response:
du -sh .
# Explanation: du calculates disk usage, -s summarizes, -h makes it human-readable.

Respond only with the command and any necessary comments/explanations according to these guidelines."#.to_string();

        let user_prompt_template = "{user_input}".to_string(); // 直接使用用户输入

        Self {
            system_prompt,
            user_prompt_template,
        }
    }
}


/// 用户界面和输出格式化相关配置。
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct UiConfig {
    /// 控制 `termichan` 输出的格式。
    pub output_format: OutputFormat,

    /// 是否在最终输出中显示 LLM 生成的命令解释（如果 LLM 提供了）。
    ///
    /// 这与 `PromptConfig` 中要求 LLM 生成解释是分开的。
    /// 此设置控制是否将 LLM 返回的解释呈现给用户。
    pub show_explanation: bool,

    /// 是否启用紧凑模式，减少输出中的垂直间距。
    pub compact_mode: bool,

    /// 是否在显示命令供用户确认或执行时使用语法高亮。
    ///
    /// 可能需要终端支持和相应的库。
    pub syntax_highlighting: bool,
}

/// 定义输出格式的枚举。
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum OutputFormat {
    /// `Plain`: 纯文本输出，不带任何特殊格式。
    Plain,
    /// `Markdown`: 使用 Markdown 格式化输出，特别是代码块。
    /// 可能不适用于直接复制粘贴命令。
    Markdown,
    /// `Rich`: 利用终端的富文本功能（如颜色、粗体）来增强可读性。
    Rich,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            output_format: OutputFormat::Rich, // 默认使用富文本以获得更好的视觉效果
            show_explanation: true, // 默认显示解释（如果提供）
            compact_mode: false, // 默认不使用紧凑模式
            syntax_highlighting: true, // 默认尝试启用语法高亮
        }
    }
}

/// 网络相关配置。
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct NetworkConfig {
    /// 网络代理服务器的 URL (可选)。
    ///
    /// 支持常见的代理协议，格式通常为 `protocol://[user:password@]host:port`。
    /// 例如:
    /// - "socks5://localhost:1080"
    /// - "http://proxy.example.com:8080"
    /// - "https://user:pass@secure.proxy.com:443"
    /// 如果为 `None`，则不使用代理，将尝试直接连接。
    /// 应用程序也可能尝试读取系统环境变量（如 `HTTP_PROXY`, `HTTPS_PROXY`）。
    pub proxy: Option<String>,

    /// 是否信任无效或自签名的 TLS/SSL 证书 (不推荐)。
    ///
    /// **安全风险**: 启用此选项会使连接容易受到中间人攻击。
    /// 仅在特殊、受控的环境下（例如连接到使用自签名证书的本地开发服务）且了解风险时启用。
    pub trust_invalid_certs: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            proxy: None, // 默认不配置代理，依赖直接连接或系统设置
            trust_invalid_certs: false, // 默认强制执行严格的证书验证
        }
    }
}