// ════════════════════════════════════════════
// MyLauncher - Rust 后端实现
// 绿色便携程序管理器
// ════════════════════════════════════════════

use tauri::{command, Emitter};
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// ─── Windows API ───
#[cfg(windows)]
use windows::{
    core::{GUID, PCWSTR, Interface, HSTRING},
    Win32::{
        Foundation::{BOOL, HWND, SIZE},
        Graphics::Gdi::*,
        Storage::FileSystem::WIN32_FIND_DATAW,
        System::Com::{CoInitializeEx, CoUninitialize, CoCreateInstance, COINIT_APARTMENTTHREADED, CLSCTX_INPROC_SERVER, IPersistFile, STGM},
        UI::{
            Shell::{ShellExecuteW, IShellItemImageFactory, IShellLinkW, SHCreateItemFromParsingName, SIIGBF_ICONONLY, SHFILEOPSTRUCTW, SHFileOperationW},
            WindowsAndMessaging::*,
        },
    },
};

// ════════════════════════════════════════════
// 数据模型
// ════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageResource {
    #[serde(rename = "type")]
    pub res_type: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverResource {
    pub enabled: bool,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub category_id: String,
    pub relative_path: String,
    pub absolute_paths: HashMap<String, String>,
    pub path_mode: String,
    pub launch_args: String,
    pub working_directory: String,
    pub window_style: String,
    pub run_as_admin: bool,
    pub url: String,
    pub icon: ImageResource,
    pub cover: CoverResource,
    pub tags: Vec<String>,
    pub notes: String,
    pub last_used: String,
    pub add_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub icon: ImageResource,
    pub view_mode: String,
    pub sort_order: i32,
    pub is_top_level: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub id: String,
    pub name: String,
    pub description: String,
    pub drive_mapping: HashMap<String, String>,
    pub is_current: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub version: String,
    /// LOGO 图标（emoji 字符或自定义图片相对路径，按 logo_icon_type 区分），默认 🪷
    #[serde(default = "default_logo_icon")]
    pub logo_icon: String,
    /// LOGO 图标类型: "emoji"（字符） | "custom"（上传的图片相对路径）
    #[serde(default = "default_logo_icon_type")]
    pub logo_icon_type: String,
    /// LOGO 文字，默认 "MyLauncher"
    #[serde(default = "default_logo_text")]
    pub logo_text: String,
    /// LOGO 图片相对路径（如 "assets/logo/xxx.png"），空表示未设置
    #[serde(default)]
    pub logo_image: String,
    /// 是否启用 LOGO 图片（启用后 LOGO 区域整体显示为图片）
    #[serde(default)]
    pub logo_image_enabled: bool,
    /// 关闭按钮行为: "tray"（隐藏到托盘） | "close"（直接退出）
    #[serde(default = "default_close_action")]
    pub close_action: String,
    /// 用户自定义 emoji 列表（在设置中粘贴添加）
    #[serde(default)]
    pub custom_emojis: Vec<String>,
    /// Favicon 获取源 API 模板列表（一行一个），支持占位符：
    /// {url} 完整链接 | {domain} 域名 | {scheme} 协议（http/https）
    /// 获取时按顺序逐个尝试，任一成功即用；列表为空时回落内置默认
    #[serde(default = "default_favicon_api_sources")]
    pub favicon_api_sources: Vec<String>,
    /// 界面主题: "light"（明亮） | "dark"（暗黑） | "system"（跟随系统）
    #[serde(default = "default_theme")]
    pub theme: String,
    /// 侧导航是否折叠隐藏
    #[serde(default)]
    pub sidebar_collapsed: bool,
    /// 竖向卡片封面宽高比例，如 "16:9" | "4:3" | "1:1" | "3:4" | "9:16"
    #[serde(default = "default_card_cover_ratio")]
    pub card_cover_ratio: String,
    /// 瀑布流封面图最大高度（px），默认 480
    #[serde(default = "default_masonry_max_height")]
    pub masonry_max_height: u32,
}

fn default_logo_icon() -> String { "🪷".to_string() }
fn default_logo_icon_type() -> String { "emoji".to_string() }
fn default_logo_text() -> String { "MyLauncher".to_string() }
fn default_close_action() -> String { "tray".to_string() }
fn default_theme() -> String { "system".to_string() }
fn default_card_cover_ratio() -> String { "16:9".to_string() }
fn default_masonry_max_height() -> u32 { 480 }

/// 默认 favicon 获取源（三个 API 模板，按序尝试）
fn default_favicon_api_sources() -> Vec<String> {
    vec![
        "https://a.favicon.im/{url}?larger=true".to_string(),
        "https://faviconsnap.com/api/favicon?url={url}".to_string(),
        "{scheme}://{domain}/favicon.ico".to_string(),
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllData {
    pub config: AppConfig,
    pub environments: Vec<Environment>,
    pub categories: Vec<Category>,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitResult {
    pub data_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExeInfo {
    pub name: String,
    pub icon_path: String,
    pub suggested_path_mode: String,
    pub relative_path: String,
    pub absolute_path: String,
    pub file_exists: bool,
    /// Steam 游戏信息（识别为 Steam 游戏时填充，否则为 None）
    #[serde(default)]
    pub steam: Option<SteamGameInfo>,
    /// 该 Steam 游戏发现多个候选 exe（后端已展开为多条，前端红色提示用户勾选）
    #[serde(default)]
    pub multi_exe: bool,
    /// 是否主程序 exe（多候选时的第一个，默认选中；单 exe 时为 true）
    #[serde(default)]
    pub is_primary: bool,
}

/// 快捷方式解析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LnkInfo {
    /// 快捷方式文件名（不含扩展名），作为条目名称
    pub name: String,
    /// 目标程序绝对路径
    pub target_path: String,
    /// 目标是否存在
    pub target_exists: bool,
    /// 工作目录（起始位置）
    pub working_directory: String,
    /// 启动参数
    pub arguments: String,
    /// 运行方式: normal | maximized | minimized
    pub window_style: String,
    /// 备注（快捷方式 comment 属性）
    pub description: String,
    /// 图标路径（提取后）
    pub icon_path: String,
    /// 建议的路径模式
    pub suggested_path_mode: String,
    /// 相对路径（如果目标在软件目录内）
    pub relative_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub results: Vec<ExeInfo>,
    pub total: usize,
}

/// Steam 游戏信息（识别 Steam 安装目录时附加在 ExeInfo 上）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamGameInfo {
    /// Steam 应用 ID（appmanifest_*.acf 的 appid）
    pub app_id: String,
    /// 游戏安装目录名（appmanifest 的 installdir，如 "Counter-Strike Global Offensive"）
    pub install_dir: String,
    /// 游戏 exe 绝对路径（在 installdir 下递归查找的候选 exe，可能为空）
    pub exe_path: String,
    /// 封面图相对路径（assets/covers/steam/{appid}.jpg，空表示未获取到；
    /// 扫描阶段不再获取，导入时由 import_scan_items 填充）
    pub cover_path: String,
    /// Steam 安装目录（导入阶段定位本地封面缓存 appcache/librarycache 用）
    #[serde(default)]
    pub steam_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathInfo {
    pub resolved: String,
    pub exists: bool,
    pub path_mode: String,
}

/// 书签链接（浏览器书签文件中的一条 <A> 记录）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkLink {
    pub title: String,
    pub url: String,
    /// 保存后的图标相对路径（如 "assets/icons/bookmarks/xxx.png"），无图标时为空
    pub icon_path: String,
}

/// 书签文件夹（可嵌套）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkFolder {
    pub title: String,
    /// 该文件夹下直属的链接（按文件内顺序）
    pub links: Vec<BookmarkLink>,
    /// 子文件夹
    pub children: Vec<BookmarkFolder>,
}

/// 书签解析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkParseResult {
    /// 解析出的所有文件夹（顶层列表，通常含一个"收藏栏"）
    pub folders: Vec<BookmarkFolder>,
    /// 直接位于根层级（不属任何文件夹）的链接
    pub root_links: Vec<BookmarkLink>,
    pub total_links: usize,
    pub total_folders: usize,
}

// ════════════════════════════════════════════
// 辅助函数
// ════════════════════════════════════════════

fn get_exe_directory() -> Result<PathBuf, String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("无法获取 exe 路径: {}", e))?;
    Ok(exe_path.parent()
        .ok_or("无法获取 exe 所在目录")?
        .to_path_buf())
}

/// 将绝对路径转为相对于 exe 目录的路径（用正斜杠），用于前端 convertFileSrc。
/// 转换失败时原样返回（也替换为正斜杠）。
fn rel_to_exe_dir(path: &Path) -> String {
    let exe_dir = get_exe_directory().unwrap_or_default();
    path.strip_prefix(&exe_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
        .replace('\\', "/")
}

/// 纯词法规范化绝对路径（消除 ..\ 与 .\ 段），不访问文件系统、不加 \\?\ 前缀。
/// SHCreateItemFromParsingName 等 shell API 不接受含 .. 的路径（0x80070057），
/// 而 std canonicalize 会产生 \\?\ 前缀路径同样不被 shell API 接受，故自实现。
fn normalize_path(path: &Path) -> PathBuf {
    let mut result: Vec<std::ffi::OsString> = Vec::new();
    let mut prefix = String::new();

    for comp in path.components() {
        use std::path::Component;
        match comp {
            Component::Prefix(p) => prefix = p.as_os_str().to_string_lossy().to_string(),
            Component::RootDir => {
                if !prefix.is_empty() && !prefix.ends_with('\\') {
                    prefix.push('\\');
                }
            }
            Component::CurDir => { /* . 段直接丢弃 */ }
            Component::ParentDir => {
                // .. 段弹出一个普通段（前缀/根不弹）
                result.pop();
            }
            Component::Normal(s) => result.push(s.to_os_string()),
        }
    }

    let mut out = PathBuf::from(prefix);
    for seg in result {
        out.push(seg);
    }
    out
}

fn get_data_dir() -> Result<PathBuf, String> {
    let exe_dir = get_exe_directory()?;
    let data_dir = exe_dir.join("data");
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)
            .map_err(|e| format!("无法创建 data 目录: {}", e))?;
    }
    Ok(data_dir)
}

fn get_assets_dir() -> Result<PathBuf, String> {
    let exe_dir = get_exe_directory()?;
    let assets_dir = exe_dir.join("assets");
    if !assets_dir.exists() {
        fs::create_dir_all(&assets_dir)
            .map_err(|e| format!("无法创建 assets 目录: {}", e))?;
    }
    Ok(assets_dir)
}

fn ensure_dir(path: &Path) -> Result<(), String> {
    if !path.exists() {
        fs::create_dir_all(path)
            .map_err(|e| format!("无法创建目录 {:?}: {}", path, e))?;
    }
    Ok(())
}

fn get_now_iso() -> String {
    chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

/// 计算从 from_dir 到 target_path 的相对路径（同盘符时可用，返回如 "..\..\Tools\app.exe"）
/// 不同盘符时返回 None。
/// 输入路径应已规范化（去除 redundant separators，使用统一分隔符）。
fn make_relative_path(from_dir: &Path, target_path: &Path) -> Option<String> {
    // Windows 下需要同盘符
    let from_components: Vec<_> = from_dir.components().collect();
    let target_components: Vec<_> = target_path.components().collect();

    // 检查盘符是否一致（Prefix 组件）
    let from_prefix = from_components.iter().find_map(|c| match c {
        std::path::Component::Prefix(p) => Some(p),
        _ => None,
    });
    let target_prefix = target_components.iter().find_map(|c| match c {
        std::path::Component::Prefix(p) => Some(p),
        _ => None,
    });

    match (&from_prefix, &target_prefix) {
        (Some(f), Some(t)) => {
            if f.kind() != t.kind() {
                return None; // 不同盘符
            }
            // 对于 VerbatimDisk 需要额外检查盘符字母
            if let (std::path::Prefix::VerbatimDisk(fc), std::path::Prefix::VerbatimDisk(tc)) = (f.kind(), t.kind()) {
                if fc != tc { return None; }
            }
            if let (std::path::Prefix::Disk(fc), std::path::Prefix::Disk(tc)) = (f.kind(), t.kind()) {
                if fc != tc { return None; }
            }
        }
        (None, None) => {} // 都没有盘符，继续
        _ => return None, // 一个有盘符一个没有
    }

    // 跳过 Prefix，从 Root 开始比较组件
    let from_parts: Vec<_> = from_components.iter()
        .filter(|c| matches!(c, std::path::Component::RootDir | std::path::Component::Normal(_)))
        .collect();
    let target_parts: Vec<_> = target_components.iter()
        .filter(|c| matches!(c, std::path::Component::RootDir | std::path::Component::Normal(_)))
        .collect();

    // 跳过 RootDir
    let from_dirs: Vec<&str> = from_parts.iter()
        .filter_map(|c| match c {
            std::path::Component::Normal(s) => s.to_str(),
            _ => None,
        })
        .collect();
    let target_dirs: Vec<&str> = target_parts.iter()
        .filter_map(|c| match c {
            std::path::Component::Normal(s) => s.to_str(),
            _ => None,
        })
        .collect();

    // 找公共前缀
    let mut common = 0;
    while common < from_dirs.len() && common < target_dirs.len() {
        if from_dirs[common].eq_ignore_ascii_case(&target_dirs[common]) {
            common += 1;
        } else {
            break;
        }
    }

    // 构建 ".." 前缀
    let up_count = from_dirs.len() - common;
    let mut parts: Vec<String> = (0..up_count).map(|_| "..".to_string()).collect();
    // 追加剩余目标组件
    for d in &target_dirs[common..] {
        parts.push(d.to_string());
    }

    if parts.is_empty() {
        return Some(".".to_string());
    }

    // Windows 下用反斜杠
    Some(parts.join("\\"))
}

fn read_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    if !path.exists() {
        return Err(format!("文件不存在: {:?}", path));
    }
    let content = fs::read_to_string(path)
        .map_err(|e| format!("读取文件失败 {:?}: {}", path, e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("解析 JSON 失败 {:?}: {}", path, e))
}

fn write_json_file<T: Serialize>(path: &Path, data: &T) -> Result<(), String> {
    let content = serde_json::to_string_pretty(data)
        .map_err(|e| format!("序列化 JSON 失败: {}", e))?;
    fs::write(path, content)
        .map_err(|e| format!("写入文件失败 {:?}: {}", path, e))?;
    Ok(())
}

// ════════════════════════════════════════════
// 初始化默认数据
// ════════════════════════════════════════════

fn default_environments() -> Vec<Environment> {
    vec![Environment {
        id: "env_001".to_string(),
        name: "本机".to_string(),
        description: "当前电脑".to_string(),
        drive_mapping: HashMap::new(),
        is_current: true,
    }]
}

fn default_categories() -> Vec<Category> {
    let make_icon = |t: &str, s: &str| ImageResource {
        res_type: t.to_string(),
        source: s.to_string(),
    };

    vec![
        Category {
            id: "cat_001".to_string(),
            name: "开发工具".to_string(),
            parent_id: None,
            icon: make_icon("builtin", "dev"),
            view_mode: "icon_grid".to_string(),
            sort_order: 1,
            is_top_level: true,
        },
        Category {
            id: "cat_002".to_string(),
            name: "游戏".to_string(),
            parent_id: None,
            icon: make_icon("builtin", "game"),
            view_mode: "vertical_card".to_string(),
            sort_order: 2,
            is_top_level: true,
        },
        Category {
            id: "cat_003".to_string(),
            name: "影音".to_string(),
            parent_id: None,
            icon: make_icon("builtin", "media"),
            view_mode: "horizontal_card".to_string(),
            sort_order: 3,
            is_top_level: true,
        },
        Category {
            id: "cat_004".to_string(),
            name: "系统工具".to_string(),
            parent_id: None,
            icon: make_icon("builtin", "system"),
            view_mode: "icon_grid".to_string(),
            sort_order: 4,
            is_top_level: true,
        },
        Category {
            id: "cat_005".to_string(),
            name: "网址收藏".to_string(),
            parent_id: None,
            icon: make_icon("builtin", "url"),
            view_mode: "icon_grid".to_string(),
            sort_order: 5,
            is_top_level: true,
        },
    ]
}

fn default_config() -> AppConfig {
    AppConfig {
        version: "1.0.0".to_string(),
        logo_icon: default_logo_icon(),
        logo_icon_type: default_logo_icon_type(),
        logo_text: default_logo_text(),
        logo_image: String::new(),
        logo_image_enabled: false,
        close_action: default_close_action(),
        custom_emojis: vec![],
        favicon_api_sources: default_favicon_api_sources(),
        theme: default_theme(),
        sidebar_collapsed: false,
        card_cover_ratio: default_card_cover_ratio(),
        masonry_max_height: default_masonry_max_height(),
    }
}

// ════════════════════════════════════════════
// Tauri Commands
// ════════════════════════════════════════════

#[command]
fn init_data_dir() -> Result<InitResult, String> {
    let data_dir = get_data_dir()?;
    let assets_dir = get_assets_dir()?;

    // 创建子目录
    for sub in &["icons/custom", "icons/extracted", "covers/custom", "covers/steam", "logo"] {
        let _ = ensure_dir(&assets_dir.join(sub));
    }

    // 初始化默认数据文件
    let env_path = data_dir.join("environments.json");
    let cat_path = data_dir.join("categories.json");
    let entry_path = data_dir.join("entries.json");
    let config_path = data_dir.join("config.json");

    if !env_path.exists() {
        let envs = default_environments();
        write_json_file(&env_path, &envs)?;
    }
    if !cat_path.exists() {
        let cats = default_categories();
        write_json_file(&cat_path, &cats)?;
    }
    if !entry_path.exists() {
        let entries: Vec<Entry> = vec![];
        write_json_file(&entry_path, &entries)?;
    }
    if !config_path.exists() {
        let config = default_config();
        write_json_file(&config_path, &config)?;
    }

    Ok(InitResult {
        data_dir: data_dir.to_string_lossy().to_string(),
    })
}

#[command]
fn load_all_data() -> Result<AllData, String> {
    let data_dir = get_data_dir()?;

    let config: AppConfig = read_json_file(&data_dir.join("config.json")).unwrap_or_else(|_| default_config());
    let environments: Vec<Environment> = read_json_file(&data_dir.join("environments.json")).unwrap_or_default();
    let categories: Vec<Category> = read_json_file(&data_dir.join("categories.json")).unwrap_or_default();
    let entries: Vec<Entry> = read_json_file(&data_dir.join("entries.json")).unwrap_or_default();

    Ok(AllData {
        config,
        environments,
        categories,
        entries,
    })
}

#[command]
fn save_entries(entries: Vec<Entry>) -> Result<(), String> {
    let data_dir = get_data_dir()?;
    write_json_file(&data_dir.join("entries.json"), &entries)
}

#[command]
fn save_categories(categories: Vec<Category>) -> Result<(), String> {
    let data_dir = get_data_dir()?;
    write_json_file(&data_dir.join("categories.json"), &categories)
}

#[command]
fn save_environments(environments: Vec<Environment>) -> Result<(), String> {
    let data_dir = get_data_dir()?;
    write_json_file(&data_dir.join("environments.json"), &environments)
}

#[command]
fn save_config(config: AppConfig) -> Result<(), String> {
    let data_dir = get_data_dir()?;
    write_json_file(&data_dir.join("config.json"), &config)
}

#[command]
fn get_exe_dir() -> Result<String, String> {
    Ok(get_exe_directory()?.to_string_lossy().to_string())
}

#[command]
fn get_now() -> String {
    get_now_iso()
}

// ─── 路径解析 ───

/// 特殊启动目标前缀（shell: URI / 协议 URI 等），不是文件路径，恒视为有效
fn is_special_uri(path: &str) -> bool {
    let p = path.trim();
    p.starts_with("shell:")
        || p.starts_with("ms-settings:")
        || p.starts_with("ms-get-started:")
        || p.starts_with("ms-contact-support:")
}

/// 裸命令名（如 taskmgr.exe）：无盘符/分隔符，依赖系统 PATH 搜索，不应拼接工作目录
fn is_bare_command(path: &str) -> bool {
    let p = path.trim();
    !p.is_empty() && !p.contains('\\') && !p.contains('/') && !p.contains(':')
}

/// 经 ShellExecuteW 启动目标：open/runas 两种动作，参数原样透传（不自动加引号），
/// 适合 explorer /select,xxx 这类对引号敏感的调用
#[cfg(windows)]
fn shell_execute_open(target: &str, args: Option<&str>, run_as_admin: bool, show_cmd: i32) -> Result<bool, String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let to_wide = |s: &str| -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    };
    let target_w = to_wide(target);
    let op_w = to_wide(if run_as_admin { "runas" } else { "open" });
    let params_w = args.map(to_wide);
    unsafe {
        let result = ShellExecuteW(
            None,
            PCWSTR(op_w.as_ptr()),
            PCWSTR(target_w.as_ptr()),
            match &params_w {
                Some(p) => PCWSTR(p.as_ptr()),
                None => PCWSTR::null(),
            },
            PCWSTR::null(),
            SHOW_WINDOW_CMD(show_cmd),
        );
        let h = result.0 as usize;
        if h <= 32 {
            return Err(format!("启动失败（错误码 {}）", h));
        }
    }
    Ok(true)
}

#[cfg(not(windows))]
fn shell_execute_open(_target: &str, _args: Option<&str>, _run_as_admin: bool, _show_cmd: i32) -> Result<bool, String> {
    Err("仅支持 Windows 平台".to_string())
}

/// 路径解析核心逻辑（相对 → 当前环境绝对 → 特殊 URI/裸命令 → fallback），供命令与图标重提取共用
fn resolve_path_inner(entry: &Entry, current_env_id: &str) -> Result<PathInfo, String> {
    let exe_dir = get_exe_directory()?;

    // 如果是网址，不需要解析路径
    if entry.entry_type == "url" {
        return Ok(PathInfo {
            resolved: entry.url.clone(),
            exists: !entry.url.is_empty(),
            path_mode: "url".to_string(),
        });
    }

    // Steam 游戏：解析游戏 exe 路径（存于 absolute_paths），存在性即有效性；
    // 启动链接存在即可视有效，exe 缺失（如未下载完）不阻断启动
    if entry.entry_type == "steam" {
        // 优先解析 exe 路径（「直接启动 exe」/「目录」定位/图标提取都依赖它）
        if let Some(abs_path) = entry.absolute_paths.get(current_env_id) {
            if !abs_path.is_empty() {
                let exists = Path::new(abs_path).exists();
                let resolved = if exists {
                    normalize_path(Path::new(abs_path)).to_string_lossy().to_string()
                } else {
                    abs_path.clone()
                };
                return Ok(PathInfo {
                    resolved,
                    exists,
                    path_mode: "absolute".to_string(),
                });
            }
        }
        // 无 exe 路径：回落到启动链接（steam:// 协议恒有效）
        return Ok(PathInfo {
            resolved: entry.url.clone(),
            exists: !entry.url.is_empty(),
            path_mode: "url".to_string(),
        });
    }

    // 特殊 URI（shell:/ms-settings: 等）：不是文件路径，恒视为存在
    // 检查相对路径和当前环境绝对路径两处
    if !entry.relative_path.is_empty() && is_special_uri(&entry.relative_path) {
        return Ok(PathInfo {
            resolved: entry.relative_path.clone(),
            exists: true,
            path_mode: "relative".to_string(),
        });
    }
    if let Some(abs_path) = entry.absolute_paths.get(current_env_id) {
        if !abs_path.is_empty() && is_special_uri(abs_path) {
            return Ok(PathInfo {
                resolved: abs_path.clone(),
                exists: true,
                path_mode: "absolute".to_string(),
            });
        }
    }

    // 裸命令名（如 taskmgr.exe / devmgmt.msc）：由系统按 PATH 搜索，
    // 不拼 exe 目录检查存在性，恒视为有效，列表不标失效
    if !entry.relative_path.is_empty() && is_bare_command(&entry.relative_path) {
        return Ok(PathInfo {
            resolved: entry.relative_path.clone(),
            exists: true,
            path_mode: "relative".to_string(),
        });
    }
    if let Some(abs_path) = entry.absolute_paths.get(current_env_id) {
        if !abs_path.is_empty() && is_bare_command(abs_path) {
            return Ok(PathInfo {
                resolved: abs_path.clone(),
                exists: true,
                path_mode: "absolute".to_string(),
            });
        }
    }

    // 1. 尝试相对路径
    if entry.path_mode == "relative" && !entry.relative_path.is_empty() {
        let full = exe_dir.join(&entry.relative_path);
        if full.exists() {
            // 规范化路径（词法消除 ..\ 等）—— SHCreateItemFromParsingName 等 shell API
            // 不接受含 .. 的路径（返回 0x80070057 参数错误）
            return Ok(PathInfo {
                resolved: normalize_path(&full).to_string_lossy().to_string(),
                exists: true,
                path_mode: "relative".to_string(),
            });
        }
    }

    // 2. 尝试当前环境的绝对路径
    if let Some(abs_path) = entry.absolute_paths.get(current_env_id) {
        if !abs_path.is_empty() && Path::new(abs_path).exists() {
            // 同上：规范化，防旧数据含 .. 形式的绝对路径
            let canonical = normalize_path(Path::new(abs_path));
            return Ok(PathInfo {
                resolved: canonical.to_string_lossy().to_string(),
                exists: true,
                path_mode: "absolute".to_string(),
            });
        }
    }

    // 3. 都不存在
    let fallback = if entry.path_mode == "relative" {
        exe_dir.join(&entry.relative_path).to_string_lossy().to_string()
    } else {
        entry.absolute_paths.get(current_env_id)
            .cloned()
            .unwrap_or_default()
    };

    Ok(PathInfo {
        resolved: fallback,
        exists: false,
        path_mode: entry.path_mode.clone(),
    })
}

#[command]
fn resolve_path(entry: Entry, current_env_id: String) -> Result<PathInfo, String> {
    resolve_path_inner(&entry, &current_env_id)
}

#[command]
fn check_paths_batch(entries: Vec<Entry>, current_env_id: String) -> Result<Vec<bool>, String> {
    let mut results = Vec::with_capacity(entries.len());
    for entry in entries {
        let info = resolve_path(entry, current_env_id.clone())?;
        results.push(info.exists);
    }
    Ok(results)
}

// ─── 启动程序 ───

#[command]
fn launch_program(entry: Entry, current_env_id: String) -> Result<bool, String> {
    // 网址：用默认浏览器打开
    if entry.entry_type == "url" {
        if entry.url.is_empty() {
            return Err("网址为空".to_string());
        }
        return shell_execute_open(entry.url.trim(), None, false, SW_SHOWNORMAL.0);
    }

    // Steam 游戏：启动链接（steam://rungameid/{appid}）经系统协议唤起 Steam 客户端
    // （「直接启动 exe」由前端构造临时 program 条目走下方的程序分支）
    if entry.entry_type == "steam" {
        if entry.url.is_empty() {
            return Err("Steam 启动链接为空".to_string());
        }
        return shell_execute_open(entry.url.trim(), None, false, SW_SHOWNORMAL.0);
    }

    // 系统功能 / APPX：目标存于 relative_path（URI 或裸命令名），直接启动
    if entry.entry_type == "system" || entry.entry_type == "appx" {
        let target = entry.relative_path.trim();
        if target.is_empty() {
            return Err("启动目标为空".to_string());
        }
        // 运行方式仅对窗口类程序有意义，这里统一按设置传参
        let show_cmd = match entry.window_style.as_str() {
            "maximized" => SW_MAXIMIZE.0,
            "minimized" => SW_MINIMIZE.0,
            _ => SW_SHOWNORMAL.0,
        };
        let args = if entry.launch_args.is_empty() { None } else { Some(entry.launch_args.as_str()) };
        return shell_execute_open(target, args, entry.run_as_admin, show_cmd);
    }

    // 程序/文件/文件夹：解析路径
    let path_info = resolve_path(entry.clone(), current_env_id)?;
    if !path_info.exists {
        return Err(format!("路径失效: {}", path_info.resolved));
    }

    let exe_path = path_info.resolved.clone();

    // 裸命令名（如 taskmgr.exe / devmgmt.msc）：直接交给 ShellExecuteW 搜索 PATH 启动
    // （.msc 由关联的 mmc.exe 打开，ShellExecuteW 原生处理）
    if is_bare_command(&exe_path) {
        let show_cmd = match entry.window_style.as_str() {
            "maximized" => SW_MAXIMIZE.0,
            "minimized" => SW_MINIMIZE.0,
            _ => SW_SHOWNORMAL.0,
        };
        let args = if entry.launch_args.is_empty() { None } else { Some(entry.launch_args.as_str()) };
        return shell_execute_open(&exe_path, args, entry.run_as_admin, show_cmd)
            .map_err(|e| format!("{}，请检查命令名是否有效", e));
    }

    #[cfg(windows)]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        let to_wide = |s: &str| -> Vec<u16> {
            OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
        };

        let operation = if entry.run_as_admin { "runas" } else { "open" };
        let params = if entry.launch_args.is_empty() {
            None
        } else {
            Some(entry.launch_args.as_str())
        };

        // 普通程序：默认工作目录为 exe 所在目录
        let working_dir = if entry.working_directory.is_empty() {
            Path::new(&exe_path).parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default()
        } else {
            entry.working_directory.clone()
        };

        let show_cmd = match entry.window_style.as_str() {
            "maximized" => SW_MAXIMIZE.0,
            "minimized" => SW_MINIMIZE.0,
            _ => SW_SHOWNORMAL.0,
        };

        let file_w = to_wide(&exe_path);
        let op_w = to_wide(operation);
        let params_w = params.map(|p| to_wide(p));
        let dir_w = to_wide(&working_dir);

        // 使用 ShellExecuteW
        unsafe {
            let result = ShellExecuteW(
                None,
                PCWSTR(op_w.as_ptr()),
                PCWSTR(file_w.as_ptr()),
            match &params_w {
                Some(p) => PCWSTR(p.as_ptr()),
                None => PCWSTR::null(),
            },
            PCWSTR(dir_w.as_ptr()),
            SHOW_WINDOW_CMD(show_cmd),
        );

            let h = result.0 as usize;
            if h <= 32 {
                // UAC 拒绝或其他错误，静默返回
                return Ok(false);
            }
        }
    }

    #[cfg(not(windows))]
    {
        let _ = exe_path;
        return Err("仅支持 Windows 平台".to_string());
    }

    Ok(true)
}

/// 打开条目所在目录（资源管理器定位并选中目标）
#[command]
fn open_in_explorer(entry: Entry, current_env_id: String) -> Result<bool, String> {
    let path_info = resolve_path(entry, current_env_id)?;
    let path = path_info.resolved.trim().to_string();

    // 特殊 URI / 裸命令名没有文件路径，无法定位
    if is_special_uri(&path) || is_bare_command(&path) || !Path::new(&path).exists() {
        return Err(format!("路径失效: {}", path));
    }

    #[cfg(windows)]
    {
        // explorer /select,"路径"：打开父目录并选中目标。
        // 必须经 ShellExecuteW 透传参数：Rust Command 传参会给含空格的参数
        // 自动整体加引号，把 /select 开关也包进去，explorer 认不出开关
        // 就退化为无参数启动（打开默认目录"文档"）
        shell_execute_open(
            "explorer.exe",
            Some(&format!("/select,\"{}\"", path)),
            false,
            SW_SHOWNORMAL.0,
        )?;
        return Ok(true);
    }

    #[cfg(not(windows))]
    {
        let _ = path;
        Err("仅支持 Windows 平台".to_string())
    }
}

// ─── 图标提取 ───

/// COM 初始化守卫：初始化 COM 并在作用域结束时调用 CoUninitialize。
/// 使用方式：`let _com = com_init()?;`
#[cfg(windows)]
struct ComGuard;
#[cfg(windows)]
impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe { CoUninitialize() }
    }
}

/// RPC_E_CHANGED_MODE 常量值
#[cfg(windows)]
const RPC_E_CHANGED_MODE: windows::core::HRESULT = windows::core::HRESULT(-2147417850i32);

/// 初始化 COM（允许重复调用，容忍 RPC_E_CHANGED_MODE）。
/// 返回 ComGuard，drop 时自动调用 CoUninitialize。
#[cfg(windows)]
fn com_init() -> Result<ComGuard, String> {
    unsafe {
        let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        if hr.0 < 0 && hr != RPC_E_CHANGED_MODE {
            return Err(format!("CoInitializeEx 失败: 0x{:08X}", hr.0 as u32));
        }
    }
    Ok(ComGuard)
}

/// 从 IShellItemImageFactory 获取指定尺寸的图标位图（BGRA → RGBA）
#[cfg(windows)]
fn get_shell_item_image(
    factory: &IShellItemImageFactory,
    size: i32,
) -> Option<image::RgbaImage> {
    use windows::Win32::Graphics::Gdi::GetObjectW;

    let h_bitmap = unsafe {
        factory.GetImage(SIZE { cx: size, cy: size }, SIIGBF_ICONONLY).ok()?
    };

    let mut bitmap = BITMAP::default();
    unsafe {
        GetObjectW(
            h_bitmap,
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut bitmap as *mut _ as *mut _),
        );
    }

    let width = bitmap.bmWidth as u32;
    let height = bitmap.bmHeight as u32;
    if width == 0 || height == 0 || bitmap.bmBits.is_null() {
        return None;
    }

    let pixel_data: &[u8] = unsafe {
        std::slice::from_raw_parts(
            bitmap.bmBits as *const u8,
            (width * height * 4) as usize,
        )
    };

    let mut pixels = pixel_data.to_vec();
    for chunk in pixels.chunks_mut(4) {
        chunk.swap(0, 2); // BGRA → RGBA
    }

    image::RgbaImage::from_raw(width, height, pixels)
}

/// 核心：提取 exe/文件的图标为 RgbaImage（COM 初始化 + 256px 图标 + 透明度回退 + 垂直翻转）。
/// 调用方负责保存 PNG。
#[cfg(windows)]
fn extract_icon_image(exe_path: &str) -> Result<image::RgbaImage, String> {
    let _com = com_init()?;

    let path_h = HSTRING::from(exe_path);
    let factory: IShellItemImageFactory = unsafe {
        SHCreateItemFromParsingName(PCWSTR(path_h.as_ptr()), None)
            .map_err(|e| format!("创建 IShellItemImageFactory 失败: {}", e))?
    };

    let mut image_buffer = get_shell_item_image(&factory, 256)
        .ok_or("无法获取图标（文件可能没有图标）")?;

    // 透明像素占比 >= 70% 时回退 48px 获取真实小图标，避免模糊
    let mut transparency: f64 = 0.0;
    let mut non_transparency: f64 = 0.0;
    for pixel in image_buffer.pixels() {
        if pixel[3] == 0 {
            transparency += 1.0;
        } else {
            non_transparency += 1.0;
        }
    }
    let total = transparency + non_transparency;
    let transparent_ratio = if total > 0.0 {
        (transparency / total * 100.0).round() as u32
    } else {
        0
    };
    if transparent_ratio >= 70 {
        if let Some(small) = get_shell_item_image(&factory, 48) {
            image_buffer = small;
        }
    }

    // BITMAP 数据是 bottom-up，需垂直翻转
    Ok(image::imageops::flip_vertical(&image_buffer))
}

/// 提取图标并保存到 assets/icons/extracted/，返回相对路径
#[cfg(windows)]
fn extract_icon_to_png(exe_path: &str) -> Result<String, String> {
    let image_buffer = extract_icon_image(exe_path)?;

    let assets_dir = get_assets_dir()?;
    let icon_dir = assets_dir.join("icons").join("extracted");
    ensure_dir(&icon_dir)?;

    let safe_name = Path::new(exe_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("icon")
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect::<String>();

    // 毫秒级时间戳：同秒内多次提取（如先单条重置又批量）不会生成同名文件互相覆盖
    let timestamp = chrono::Local::now().format("%H%M%S%3f").to_string();
    let icon_file = icon_dir.join(format!("{}_{}.png", safe_name, timestamp));

    image_buffer
        .save(&icon_file)
        .map_err(|e| format!("保存图标失败: {}", e))?;

    Ok(rel_to_exe_dir(&icon_file))
}

/// 提取图标并保存到指定路径
#[cfg(windows)]
fn extract_icon_to_path(exe_path: &str, icon_file: &Path) -> Result<(), String> {
    let image_buffer = extract_icon_image(exe_path)?;
    image_buffer
        .save(icon_file)
        .map_err(|e| format!("保存图标失败: {}", e))?;
    Ok(())
}

#[cfg(not(windows))]
fn extract_icon_to_png(_exe_path: &str) -> Result<String, String> {
    Err("仅支持 Windows 平台".to_string())
}

#[cfg(not(windows))]
fn extract_icon_to_path(_exe_path: &str, _icon_file: &Path) -> Result<(), String> {
    Err("仅支持 Windows 平台".to_string())
}

#[command]
fn get_exe_info(exe_path: String) -> Result<ExeInfo, String> {
    let path = Path::new(&exe_path);
    if !path.exists() {
        return Ok(ExeInfo {
            name: String::new(),
            icon_path: String::new(),
            suggested_path_mode: "absolute".to_string(),
            relative_path: String::new(),
            absolute_path: exe_path.clone(),
            file_exists: false,
            steam: None,
            multi_exe: false,
            is_primary: true,
        });
    }

    let name = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("未知")
        .to_string();

    let exe_dir = get_exe_directory()?;

    // 同盘符时建议相对路径（..\..\ 形式），不同盘符时用绝对路径
    let (suggested_mode, rel_path) = if let Some(rel) = make_relative_path(&exe_dir, path) {
        ("relative", rel)
    } else {
        ("absolute", String::new())
    };

    // 尝试提取图标
    let icon_path = extract_icon_to_png(&exe_path).unwrap_or_default();

    Ok(ExeInfo {
        name,
        icon_path,
        suggested_path_mode: suggested_mode.to_string(),
        relative_path: rel_path,
        absolute_path: exe_path.clone(),
        file_exists: true,
        steam: None,
        multi_exe: false,
        is_primary: true,
    })
}

// ─── 文件/文件夹信息 ───

/// 文件/文件夹信息（用于添加文件夹和文件条目）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub entry_type: String,   // "folder" | "file"
    pub icon_path: String,    // 提取的图标相对路径
    pub suggested_path_mode: String,
    pub relative_path: String,
    pub absolute_path: String,
    pub file_exists: bool,
}

/// 获取文件/文件夹信息，图标按类型/扩展名缓存去重
#[command]
fn get_file_info(file_path: String) -> Result<FileInfo, String> {
    let path = Path::new(&file_path);
    if !path.exists() {
        return Ok(FileInfo {
            name: String::new(),
            entry_type: "file".to_string(),
            icon_path: String::new(),
            suggested_path_mode: "absolute".to_string(),
            relative_path: String::new(),
            absolute_path: file_path.clone(),
            file_exists: false,
        });
    }

    let is_folder = path.is_dir();
    let entry_type = if is_folder { "folder" } else { "file" };

    let name = path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(if is_folder { "文件夹" } else { "文件" })
        .to_string();

    let exe_dir = get_exe_directory()?;
    let (suggested_mode, rel_path) = if let Some(rel) = make_relative_path(&exe_dir, path) {
        ("relative", rel)
    } else {
        ("absolute", String::new())
    };

    // 图标缓存：文件夹共用一个图标，文件按扩展名共用
    let assets_dir = get_assets_dir()?;
    let cache_dir = assets_dir.join("icons").join("file_cache");
    ensure_dir(&cache_dir)?;

    let cache_key = if is_folder {
        "folder".to_string()
    } else {
        let ext = path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("none")
            .to_lowercase();
        format!("ext_{}", ext)
    };
    let icon_file = cache_dir.join(format!("{}.png", cache_key));

    // 已有缓存则直接用，没有则提取
    let icon_path = if icon_file.exists() {
        rel_to_exe_dir(&icon_file)
    } else {
        match extract_icon_to_path(&file_path, &icon_file) {
            Ok(()) => rel_to_exe_dir(&icon_file),
            Err(_) => String::new(),
        }
    };

    Ok(FileInfo {
        name,
        entry_type: entry_type.to_string(),
        icon_path,
        suggested_path_mode: suggested_mode.to_string(),
        relative_path: rel_path,
        absolute_path: file_path.clone(),
        file_exists: true,
    })
}

// ─── 快捷方式解析 ───

/// Windows 快捷方式解析核心：用 IShellLinkW + IPersistFile 读取 .lnk 属性
#[cfg(windows)]
fn resolve_lnk_file(lnk_path: &str) -> Result<LnkInfo, String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let to_wide = |s: &str| -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    };

    // 1. 初始化 COM
    let _com = com_init()?;

    // 2. 创建 IShellLinkW 实例
    // ShellLink CLSID: {00021401-0000-0000-C000-000000000046}
    const SHELL_LINK_CLSID: GUID = GUID::from_u128(0x00021401_0000_0000_C000_000000000046);
    let shell_link: IShellLinkW = unsafe {
        CoCreateInstance(
            &SHELL_LINK_CLSID,
            None,
            CLSCTX_INPROC_SERVER,
        )
        .map_err(|e| format!("创建 IShellLinkW 失败: {}", e))?
    };

    // 3. 用 IPersistFile 加载 .lnk 文件
    let persist_file: IPersistFile = shell_link
        .cast()
        .map_err(|e| format!("获取 IPersistFile 失败: {}", e))?;

    let lnk_w = to_wide(lnk_path);
    unsafe {
        persist_file.Load(PCWSTR(lnk_w.as_ptr()), STGM(0))
            .map_err(|e| format!("加载 .lnk 失败: {}", e))?;
    }

    // 4. 读取各属性
    let exe_dir = get_exe_directory()?;

    // 目标路径
    let mut target_buf = [0u16; 260];
    let mut find_data = WIN32_FIND_DATAW::default();
    unsafe {
        shell_link.GetPath(&mut target_buf, &mut find_data, 0)
            .map_err(|e| format!("GetPath 失败: {}", e))?;
    }
    let target_path = String::from_utf16_lossy(
        &target_buf[..target_buf.iter().position(|&c| c == 0).unwrap_or(0)]
    );

    // 工作目录
    let mut work_dir_buf = [0u16; 260];
    unsafe {
        shell_link.GetWorkingDirectory(&mut work_dir_buf)
            .ok();
    }
    let working_directory = String::from_utf16_lossy(
        &work_dir_buf[..work_dir_buf.iter().position(|&c| c == 0).unwrap_or(0)]
    );

    // 启动参数
    let mut args_buf = [0u16; 512];
    unsafe {
        shell_link.GetArguments(&mut args_buf)
            .ok();
    }
    let arguments = String::from_utf16_lossy(
        &args_buf[..args_buf.iter().position(|&c| c == 0).unwrap_or(0)]
    );

    // 运行方式
    let show_cmd = unsafe { shell_link.GetShowCmd() }.unwrap_or(SHOW_WINDOW_CMD(1));
    let window_style = match show_cmd.0 {
        3 => "maximized",   // SW_MAXIMIZE
        2 => "minimized",   // SW_MINIMIZE
        _ => "normal",
    };

    // 备注（description）
    let mut desc_buf = [0u16; 1024];
    unsafe {
        shell_link.GetDescription(&mut desc_buf)
            .ok();
    }
    let description = String::from_utf16_lossy(
        &desc_buf[..desc_buf.iter().position(|&c| c == 0).unwrap_or(0)]
    );

    // 快捷方式文件名（不含扩展名）
    let lnk_name = Path::new(lnk_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("快捷方式")
        .to_string();

    // 目标是否存在
    let target_exists = Path::new(&target_path).exists();

    // 同盘符时建议相对路径
    let target_path_obj = Path::new(&target_path);
    let (suggested_mode, rel_path) = if !target_path.is_empty() {
        if let Some(rel) = make_relative_path(&exe_dir, target_path_obj) {
            ("relative", rel)
        } else {
            ("absolute", String::new())
        }
    } else {
        ("absolute", String::new())
    };

    // 提取目标程序图标
    let icon_path = if target_exists {
        extract_icon_to_png(&target_path).unwrap_or_default()
    } else {
        String::new()
    };

    Ok(LnkInfo {
        name: lnk_name,
        target_path,
        target_exists,
        working_directory,
        arguments,
        window_style: window_style.to_string(),
        description,
        icon_path,
        suggested_path_mode: suggested_mode.to_string(),
        relative_path: rel_path,
    })
}

#[cfg(not(windows))]
fn resolve_lnk_file(_lnk_path: &str) -> Result<LnkInfo, String> {
    Err("仅支持 Windows 平台".to_string())
}

#[command]
fn resolve_lnk(lnk_path: String) -> Result<LnkInfo, String> {
    let path = Path::new(&lnk_path);
    if !path.exists() {
        return Err(format!("文件不存在: {}", lnk_path));
    }
    let ext = path.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    if ext != "lnk" {
        return Err(format!("仅支持 .lnk 快捷方式文件，当前: .{}", ext));
    }
    resolve_lnk_file(&lnk_path)
}

// ─── APPX / UWP 应用枚举 ───

/// APPX 应用信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppxAppInfo {
    /// 显示名称
    pub name: String,
    /// shell:AppsFolder\{family_name}!{app_id}
    pub shell_uri: String,
    /// 应用唯一标识（PackageFullName!AppId），用于图标缓存命名
    pub app_key: String,
    /// 提取的图标相对路径（assets/icons/extracted/ 下）
    pub icon_path: String,
}

/// 通过 PowerShell Get-StartApps 枚举开始菜单应用（含 APPX/UWP + 传统快捷方式）
/// Get-StartApps 返回 AppID（AUMID）和 Name，AUMID 形如
/// "Microsoft.WindowsCalculator_8wekyb3d8bbwe!App" 或 "C:\...\xxx.lnk"
#[cfg(windows)]
fn enumerate_appx_apps() -> Result<Vec<AppxAppInfo>, String> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command",
               "[Console]::OutputEncoding=[System.Text.Encoding]::UTF8; Get-StartApps | Select-Object Name, AppID | ConvertTo-Json -Compress"])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .map_err(|e| format!("启动 PowerShell 失败: {}", e))?;

    if !output.status.success() {
        return Err(format!("Get-StartApps 失败: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let json: serde_json::Value = match serde_json::from_str(stdout.trim()) {
        Ok(v) => v,
        Err(e) => {
            // UTF-8 解析失败时回退 GBK 解码（旧系统控制台编码）
            let decoded = encoding_rs::GBK.decode(&output.stdout).0.to_string();
            serde_json::from_str(decoded.trim())
                .map_err(|e2| format!("解析 Get-StartApps 输出失败: UTF8({}), GBK({})", e, e2))?
        }
    };

    // 单个结果时是对象，多个时是数组
    let items: Vec<serde_json::Value> = match json {
        serde_json::Value::Array(arr) => arr,
        obj @ serde_json::Value::Object(_) => vec![obj],
        _ => Vec::new(),
    };

    let mut apps = Vec::new();
    for item in items {
        let name = item.get("Name").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
        let appid = item.get("AppID").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
        if name.is_empty() || appid.is_empty() {
            continue;
        }
        // 只要 AUMID 格式（含 ! 的即 APPX/UWP 应用），排除 .lnk 路径
        if !appid.contains('!') {
            continue;
        }

        let shell_uri = format!("shell:AppsFolder\\{}", appid);
        let app_key: String = appid
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
            .collect();

        apps.push(AppxAppInfo {
            name,
            shell_uri,
            app_key,
            icon_path: String::new(),
        });
    }

    // 按名称排序
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(apps)
}

#[cfg(not(windows))]
fn enumerate_appx_apps() -> Result<Vec<AppxAppInfo>, String> {
    Err("仅支持 Windows 平台".to_string())
}

/// 枚举 APPX 应用（不提取图标，前端按需批量提取）
#[command]
fn list_appx_apps() -> Result<Vec<AppxAppInfo>, String> {
    enumerate_appx_apps()
}

// ─── 图标重新提取 ───

/// 图标重提取结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconRefreshResult {
    pub success: bool,
    /// 新图标相对路径（失败时为空）
    pub icon_path: String,
    /// 失败原因（成功时为空）
    pub message: String,
}

impl IconRefreshResult {
    fn ok(path: String) -> Self {
        IconRefreshResult { success: true, icon_path: path, message: String::new() }
    }
    fn err(msg: String) -> Self {
        IconRefreshResult { success: false, icon_path: String::new(), message: msg }
    }
}

/// 按条目类型分派图标重提取，返回新的图标相对路径（assets/ 相对路径）。
/// 不修改 JSON 数据（由前端保存），仅生成新图标文件。
fn refresh_icon_inner(entry: &Entry, current_env_id: &str) -> Result<String, String> {
    match entry.entry_type.as_str() {
        "steam" => {
            // Steam 游戏：与 program 一致，从解析出的游戏 exe 提取图标
            // （resolve_path_inner 对 steam 条目优先解析 absolute_paths 中的 exe 路径）
            let info = resolve_path_inner(entry, current_env_id)?;
            if !info.exists {
                return Err(format!("游戏主程序不存在: {}", info.resolved));
            }
            extract_icon_to_png(&info.resolved)
        }
        "program" | "folder" | "file" => {
            // 复用 resolve_path 的解析逻辑（相对 → 当前环境绝对 → fallback）
            let info = resolve_path_inner(entry, current_env_id)?;
            if !info.exists {
                return Err(format!("目标不存在: {}", info.resolved));
            }
            if entry.entry_type == "program" {
                extract_icon_to_png(&info.resolved)
            } else {
                // 文件夹/文件：沿用 get_file_info 的缓存逻辑（文件夹共用 folder.png，文件按扩展名缓存）
                let path = Path::new(&info.resolved);
                let cache_key = if path.is_dir() {
                    "folder".to_string()
                } else {
                    let ext = path.extension()
                        .and_then(|s| s.to_str())
                        .unwrap_or("none")
                        .to_lowercase();
                    format!("ext_{}", ext)
                };
                let cache_dir = get_assets_dir()?.join("icons").join("file_cache");
                ensure_dir(&cache_dir)?;
                let icon_file = cache_dir.join(format!("{}.png", cache_key));
                extract_icon_to_path(&info.resolved, &icon_file)?;
                Ok(rel_to_exe_dir(&icon_file))
            }
        }
        "url" => {
            // 网址类型：多源 favicon 链（a.favicon.im → faviconsnap → 原网址 /favicon.ico，各 2 秒超时）
            // 全部失败返回 Err，由调用方跳过该条目（保持原图标不变）
            // 重置语义 = force：绕过 domain 缓存强制重新下载最新 favicon
            if entry.url.trim().is_empty() {
                return Err("网址为空".to_string());
            }
            fetch_favicon_inner(&entry.url, true)
        }
        "appx" => {
            // 目标存于 relative_path（shell:AppsFolder\{family}!{appid}），app_key 与枚举时推导一致
            let shell_uri = entry.relative_path.trim();
            if shell_uri.is_empty() {
                return Err("APPX 启动目标为空".to_string());
            }
            let appid = shell_uri
                .strip_prefix("shell:AppsFolder\\")
                .unwrap_or(shell_uri);
            let app_key: String = appid
                .chars()
                .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
                .collect();
            // 重置语义 = 强制重新提取：先删旧缓存文件，inner 检测不到缓存即走真实提取路径
            let old_cache = get_assets_dir()?
                .join("icons")
                .join("extracted")
                .join(format!("appx_{}.png", app_key));
            let _ = fs::remove_file(&old_cache);
            extract_appx_icon_inner(&app_key, shell_uri)
        }
        // system 类型为 emoji 图标，无需提取
        "system" => Err("系统功能使用 Emoji 图标，无需重新提取".to_string()),
        other => Err(format!("未知类型: {}", other)),
    }
}

/// extract_appx_icon 的内部封装（供命令与重提取共用，绕过缓存时先删旧缓存文件）
fn extract_appx_icon_inner(app_key: &str, shell_uri: &str) -> Result<String, String> {
    let assets_dir = get_assets_dir()?;
    let icon_dir = assets_dir.join("icons").join("extracted");
    ensure_dir(&icon_dir)?;
    let icon_file = icon_dir.join(format!("appx_{}.png", app_key));

    // 已有缓存直接返回（重提取场景下旧图标仍有效，避免重复提取开销）
    if icon_file.exists() {
        return Ok(rel_to_exe_dir(&icon_file));
    }

    extract_icon_to_path(shell_uri, &icon_file)?;
    Ok(rel_to_exe_dir(&icon_file))
}

/// 重新提取条目图标（单条）
#[command]
fn refresh_entry_icon(entry: Entry, current_env_id: String) -> Result<IconRefreshResult, String> {
    match refresh_icon_inner(&entry, &current_env_id) {
        Ok(path) => Ok(IconRefreshResult::ok(path)),
        Err(e) => Ok(IconRefreshResult::err(e)),
    }
}

// ─── 批量重新提取图标（多线程 + 进度事件） ───

/// 批量重置图标的单条结果
#[derive(Debug, Clone, Serialize)]
struct BatchIconItemResult {
    entry_id: String,
    entry_name: String,
    entry_type: String,
    /// success / skipped / failed
    status: String,
    /// 跳过或失败原因（成功时为空）
    message: String,
    /// 成功时的新图标相对路径（assets/ 相对路径）
    icon_path: String,
    /// 在输入列表中的原始下标（用于结果排序，不序列化）
    #[serde(skip_serializing)]
    entry_index: usize,
}

/// 批量重置图标的整体统计
#[derive(Debug, Clone, Serialize)]
struct BatchIconSummary {
    total: usize,
    success: usize,
    skipped: usize,
    failed: usize,
    results: Vec<BatchIconItemResult>,
}

/// 批量重新提取图标（后端 std::thread 多线程，逐条通过事件推送进度）。
/// 条目全部处理完毕后返回整体统计。
/// 规则与单条 refresh_entry_icon 一致：system 类型直接跳过；
/// url 类型 favicon 链全失败记为 skipped（保持原图标不变）；其余类型失败记 failed。
/// 事件：batch-icon-progress（单条完成，payload 含 done/total 与单条结果）。
#[command]
fn batch_refresh_icons(
    app: tauri::AppHandle,
    entries: Vec<Entry>,
    current_env_id: String,
) -> Result<BatchIconSummary, String> {
    use std::sync::{Arc, Mutex};
    use std::sync::atomic::{AtomicUsize, Ordering};

    let total = entries.len();
    if total == 0 {
        return Ok(BatchIconSummary { total: 0, success: 0, skipped: 0, failed: 0, results: Vec::new() });
    }

    let indexed: Vec<(usize, Entry)> = entries.into_iter().enumerate().collect();
    let results: Arc<Mutex<Vec<BatchIconItemResult>>> = Arc::new(Mutex::new(Vec::with_capacity(total)));
    let done_count = Arc::new(AtomicUsize::new(0));

    // 线程数：条目数与 8 取小（COM/Shell API 图标提取较重，过多线程收益递减）
    let thread_num = total.min(8).max(1);
    let chunk_size = (total + thread_num - 1) / thread_num;

    let mut handles = Vec::new();
    for chunk in indexed.chunks(chunk_size) {
        let results = Arc::clone(&results);
        let done_count = Arc::clone(&done_count);
        let env_id = current_env_id.clone();
        let app = app.clone();
        let chunk_vec = chunk.to_vec();
        handles.push(std::thread::spawn(move || {
            for (idx, entry) in chunk_vec {
                // COM 初始化：extract_icon_image 内部每次调用 com_init()，
                // 各工作线程独立初始化，无需额外处理
                let (status, message, icon_path) = if entry.entry_type == "system" {
                    ("skipped".to_string(), "系统功能使用 Emoji 图标，无需提取".to_string(), String::new())
                } else {
                    match refresh_icon_inner(&entry, &env_id) {
                        Ok(path) => ("success".to_string(), String::new(), path),
                        Err(e) if entry.entry_type == "url" =>
                            // url 类型 favicon 链全失败：跳过，保持原图标不变
                            ("skipped".to_string(), e, String::new()),
                        Err(e) => ("failed".to_string(), e, String::new()),
                    }
                };
                let item = BatchIconItemResult {
                    entry_id: entry.id.clone(),
                    entry_name: entry.name.clone(),
                    entry_type: entry.entry_type.clone(),
                    status: status.clone(),
                    message: message.clone(),
                    icon_path: icon_path.clone(),
                    entry_index: idx,
                };
                {
                    let mut list = results.lock().unwrap();
                    list.push(item);
                }
                let done = done_count.fetch_add(1, Ordering::SeqCst) + 1;
                // 逐条推送进度（前端弹窗列表实时展示）
                let _ = app.emit("batch-icon-progress", serde_json::json!({
                    "done": done,
                    "total": total,
                    "entry_id": entry.id,
                    "entry_name": entry.name,
                    "entry_type": entry.entry_type,
                    "status": status,
                    "message": message,
                    "icon_path": icon_path,
                }));
            }
        }));
    }
    for h in handles {
        let _ = h.join();
    }

    let mut sorted = Arc::try_unwrap(results)
        .map_err(|_| "结果收集失败".to_string())?
        .into_inner()
        .unwrap();
    // 恢复与输入一致的顺序
    sorted.sort_by_key(|r| r.entry_index);

    let success = sorted.iter().filter(|r| r.status == "success").count();
    let skipped = sorted.iter().filter(|r| r.status == "skipped").count();
    let failed = sorted.iter().filter(|r| r.status == "failed").count();

    Ok(BatchIconSummary { total, success, skipped, failed, results: sorted })
}

/// 为 APPX 应用提取图标（从 shell:AppsFolder URI），返回图标相对路径
#[command]
fn extract_appx_icon(app_key: String, shell_uri: String) -> Result<String, String> {
    let assets_dir = get_assets_dir()?;
    let icon_dir = assets_dir.join("icons").join("extracted");
    ensure_dir(&icon_dir)?;

    let icon_file = icon_dir.join(format!("appx_{}.png", app_key));

    // 已有缓存直接返回
    if icon_file.exists() {
        return Ok(rel_to_exe_dir(&icon_file));
    }

    extract_icon_to_path(&shell_uri, &icon_file)?;
    Ok(rel_to_exe_dir(&icon_file))
}

// ─── 文件夹扫描 ───

/// 判断指定目录是否为 Steam 安装目录（存在 steamapps 子目录）
fn is_steam_dir(path: &Path) -> bool {
    path.join("steamapps").is_dir()
}

/// 读取 steamapps 目录下的所有 appmanifest_*.acf，解析出 Steam 游戏列表。
/// 每个条目包含 appid/name/installdir（解析失败的行跳过）。
fn read_steam_manifests(steamapps_dir: &Path) -> Vec<(String, String, String)> {
    let mut games = Vec::new();
    let entries = match std::fs::read_dir(steamapps_dir) {
        Ok(e) => e,
        Err(_) => return games,
    };

    for item in entries.flatten() {
        let path = item.path();
        let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if !fname.starts_with("appmanifest_") || !fname.ends_with(".acf") {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let mut appid = String::new();
        let mut name = String::new();
        let mut installdir = String::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() { continue; }
            if let Some(v) = extract_acf_value(line, "appid") { appid = v.to_string(); }
            else if let Some(v) = extract_acf_value(line, "name") { name = v.to_string(); }
            else if let Some(v) = extract_acf_value(line, "installdir") { installdir = v.to_string(); }
        }
        if !appid.is_empty() && !name.is_empty() && !installdir.is_empty() {
            games.push((appid, name, installdir));
        }
    }
    games
}

/// 从 acf 行中提取指定 key 的字符串值（支持 "key" "value" 形式）
fn extract_acf_value(line: &str, key: &str) -> Option<String> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix(&format!("\"{}\"", key))?;
    let rest = rest.trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// 在游戏安装目录下递归查找候选 exe（跳过常见无意义文件，如 unins*.exe /
/// Redist 目录 / win 开头文件）。返回全部候选，第一个视为主程序。
fn find_game_exes(install_dir: &Path) -> Vec<String> {
    let mut found = Vec::new();
    for entry in walkdir::WalkDir::new(install_dir)
        .max_depth(4)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() { continue; }
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
        if ext != "exe" { continue; }
        let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
        if fname.starts_with("unins") { continue; }
        if fname.starts_with("win") && fname.ends_with(".exe") { continue; }
        // 跳过 redist/vcredist 等常见非游戏主程序
        let rel = path.strip_prefix(install_dir).unwrap_or(path);
        let rel_lower = rel.to_string_lossy().to_lowercase();
        if rel_lower.contains("redist") || rel_lower.contains("vcredist") || rel_lower.contains("directx") {
            continue;
        }
        found.push(path.to_string_lossy().to_string());
    }
    found
}

/// 尝试将 Steam 本地缓存的 header 图（appcache/librarycache/{appid}_header.jpg）复制到
/// assets/covers/steam/{appid}.jpg；缓存缺失时尝试从 CDN 下载（2 秒超时）。
/// 全部失败返回空字符串（条目不设封面）。
fn get_steam_cover(steam_install_dir: &Path, appid: &str) -> String {
    // 0. 目标已存在直接复用（同一游戏多 exe 并发导入时避免并发写同一文件失败）
    if let Ok(assets_dir) = get_assets_dir() {
        let target = assets_dir.join("covers").join("steam").join(format!("{}.jpg", appid));
        if target.is_file() {
            return rel_to_exe_dir(&target);
        }
    }

    // 1. 优先复制 Steam 本地缓存（用户明确要求优先用本地缓存）
    let cache_dir = steam_install_dir.join("appcache").join("librarycache");
    let cache_file = cache_dir.join(format!("{}_header.jpg", appid));
    if cache_file.is_file() {
        if let Ok(rel) = copy_steam_cover_to_assets(&cache_file, appid) {
            return rel;
        }
    }

    // 2. 本地缓存缺失 → 从 CDN 下载（与 favicon 相同的 2 秒超时客户端）
    let url = format!("https://cdn.cloudflare.steamstatic.com/steam/apps/{}/header.jpg", appid);
    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
    {
        Ok(c) => c,
        Err(_) => return String::new(),
    };
    let bytes = match client.get(&url).send().and_then(|r| r.bytes()) {
        Ok(b) => b,
        Err(_) => return String::new(),
    };
    if bytes.is_empty() || !bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return String::new();
    }

    let assets_dir = match get_assets_dir() {
        Ok(d) => d,
        Err(_) => return String::new(),
    };
    let steam_dir = assets_dir.join("covers").join("steam");
    if ensure_dir(&steam_dir).is_err() {
        return String::new();
    }
    let target = steam_dir.join(format!("{}.jpg", appid));
    if std::fs::write(&target, &bytes).is_ok() {
        rel_to_exe_dir(&target)
    } else {
        String::new()
    }
}

/// 将本地 header 图片复制到 assets/covers/steam/{appid}.jpg，返回相对路径
fn copy_steam_cover_to_assets(src: &Path, appid: &str) -> Result<String, String> {
    let assets_dir = get_assets_dir()?;
    let steam_dir = assets_dir.join("covers").join("steam");
    ensure_dir(&steam_dir)?;
    let target = steam_dir.join(format!("{}.jpg", appid));
    std::fs::copy(src, &target)
        .map_err(|e| format!("复制封面失败: {}", e))?;
    Ok(rel_to_exe_dir(&target))
}

/// 从注册表 HKCU\Software\Valve\Steam 读取 Steam 安装路径（SteamPath 值）。
/// 未安装 Steam 或读取失败返回 None。
#[cfg(windows)]
fn get_steam_install_dir_from_registry() -> Option<PathBuf> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::System::Registry::{RegCloseKey, RegOpenKeyW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER};

    fn to_wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    unsafe {
        let sub_key = to_wide(r"Software\Valve\Steam");
        let value_name = to_wide("SteamPath");
        let mut key = HKEY::default();
        // 只读打开（不创建）：未安装 Steam 时返回失败
        if RegOpenKeyW(HKEY_CURRENT_USER, PCWSTR(sub_key.as_ptr()), &mut key).is_err() {
            return None;
        }
        let mut buf_len: u32 = 0;
        let query = RegQueryValueExW(
            key,
            PCWSTR(value_name.as_ptr()),
            None,
            None,
            None,
            Some(&mut buf_len),
        );
        if query.is_err() || buf_len == 0 {
            let _ = RegCloseKey(key);
            return None;
        }
        let mut buf = vec![0u8; buf_len as usize];
        let result = RegQueryValueExW(
            key,
            PCWSTR(value_name.as_ptr()),
            None,
            None,
            Some(buf.as_mut_ptr()),
            Some(&mut buf_len),
        );
        let _ = RegCloseKey(key);
        if result.is_err() {
            return None;
        }
        // 字节 → UTF-16 字符串（去掉结尾 NUL）
        let words: Vec<u16> = buf
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        let s = String::from_utf16_lossy(&words);
        let s = s.trim_end_matches('\0');
        if s.is_empty() { None } else { Some(PathBuf::from(s)) }
    }
}

#[cfg(not(windows))]
fn get_steam_install_dir_from_registry() -> Option<PathBuf> {
    None
}

/// 解析主 Steam 目录下 steamapps/libraryfolders.vdf，返回所有库的 steamapps 目录列表
/// （含主库）。文件缺失或解析失败时仅返回主库。
fn read_steam_library_dirs(steam_dir: &Path) -> Vec<PathBuf> {
    let main_steamapps = steam_dir.join("steamapps");
    let mut libs = vec![main_steamapps.clone()];

    let vdf_path = main_steamapps.join("libraryfolders.vdf");
    let content = match std::fs::read_to_string(&vdf_path) {
        Ok(c) => c,
        Err(_) => return libs,
    };
    // 逐行提取 "path" 值（vdf KeyValues 格式，与 acf 相同的行级 key-value）
    // 路径中的 \\ 转义还原为 \
    for line in content.lines() {
        if let Some(p) = extract_acf_value(line.trim(), "path") {
            if p.is_empty() { continue; }
            let dir = PathBuf::from(p.replace("\\\\", "\\")).join("steamapps");
            // 去重：主库通常也出现在 vdf 中
            if !libs.contains(&dir) {
                libs.push(dir);
            }
        }
    }
    libs
}

/// 按 Steam 应用 ID 查找已安装游戏的扫描结果
#[derive(Debug, Clone, Serialize)]
struct SteamAppScanResult {
    /// 游戏名称（appmanifest 的 name）
    pub name: String,
    /// 游戏 exe 绝对路径（安装目录下递归查找，可能为空）
    pub exe_path: String,
    /// 封面图相对路径（assets/covers/steam/{appid}.jpg，空表示未获取到）
    pub cover_path: String,
    /// 图标相对路径（从游戏 exe 提取，可能为空）
    pub icon_path: String,
}

/// 按 Steam 应用 ID 扫描单个已安装游戏。
/// 流程：注册表定位 Steam → 解析 libraryfolders.vdf 遍历所有库 →
/// 各库找 steamapps/appmanifest_{appid}.acf → 回填名称/exe/封面/图标。
/// 找不到 manifest 说明该 ID 游戏不存在或未安装。
#[command]
fn scan_steam_app(app_id: String) -> Result<SteamAppScanResult, String> {
    let app_id = app_id.trim().to_string();
    if app_id.is_empty() {
        return Err("请输入 Steam 游戏 ID".to_string());
    }

    let steam_dir = get_steam_install_dir_from_registry()
        .ok_or_else(|| "未检测到 Steam 客户端（注册表 HKCU\\Software\\Valve\\Steam）".to_string())?;
    if !steam_dir.is_dir() {
        return Err(format!("Steam 目录不存在: {}", steam_dir.display()));
    }

    // 遍历所有库找 appmanifest_{appid}.acf
    let manifest_file = format!("appmanifest_{}.acf", app_id);
    for lib_steamapps in read_steam_library_dirs(&steam_dir) {
        let manifest_path = lib_steamapps.join(&manifest_file);
        let content = match std::fs::read_to_string(&manifest_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let mut name = String::new();
        let mut installdir = String::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() { continue; }
            if let Some(v) = extract_acf_value(line, "name") { name = v; }
            else if let Some(v) = extract_acf_value(line, "installdir") { installdir = v; }
        }
        if name.is_empty() || installdir.is_empty() {
            continue;
        }

        // 解析游戏信息（exe/封面/图标）——单游戏流程量小，保留即时提取
        let install_dir = lib_steamapps.join("common").join(&installdir);
        if !install_dir.is_dir() {
            return Err(format!("游戏目录不存在: {}", install_dir.display()));
        }
        let game_exe = find_game_exes(&install_dir).into_iter().next().unwrap_or_default();
        let cover_path = get_steam_cover(&steam_dir, &app_id);
        let icon_path = if !game_exe.is_empty() {
            extract_icon_to_png(&game_exe).unwrap_or_default()
        } else {
            String::new()
        };

        return Ok(SteamAppScanResult { name, exe_path: game_exe, cover_path, icon_path });
    }

    Err(format!("游戏 ID {} 不存在或未安装（所有 Steam 库中均未找到 appmanifest_{}.acf）", app_id, app_id))
}

/// 扫描目录（普通文件夹/Steam 安装目录）。
/// 扫描阶段不提取图标、不下载封面（保证速度），仅收集文件清单；
/// 图标/封面等重操作移到导入命令 import_scan_items 中执行。
/// 异步执行并推送 scan-progress 事件（前端进度弹窗实时展示）：
///   - phase="walking"  ：遍历文件阶段，payload 含 found（已发现的 exe 数）
///   - phase="icons"    ：解析阶段（Steam 游戏清单），payload 含 done/total
/// 事件名：scan-progress
#[command]
async fn scan_directory(app: tauri::AppHandle, dir_path: String, max_depth: Option<usize>) -> Result<ScanResult, String> {
    let path = Path::new(&dir_path);
    if !path.exists() || !path.is_dir() {
        return Err(format!("目录不存在: {}", dir_path));
    }

    // Steam 安装目录：直接解析 appmanifest 生成游戏条目（不递归扫 exe）
    if is_steam_dir(path) {
        let _ = app.emit("scan-progress", serde_json::json!({
            "phase": "walking",
            "found": 0,
            "done": 0,
            "total": 0,
        }));
        return scan_steam_games(&app, path);
    }

    let depth = max_depth.unwrap_or(3);
    let exe_dir = get_exe_directory()?;
    let mut results = Vec::new();

    // 阶段一：遍历目录收集 exe 文件（全程推送 found 计数）
    let mut walk_paths = Vec::new();
    for entry in walkdir::WalkDir::new(&dir_path)
        .max_depth(depth)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
        if ext != "exe" {
            continue;
        }
        let abs_path = path.to_string_lossy().to_string();
        walk_paths.push(abs_path);
        // 每发现一个 exe 推送一次（前端进度文字「正在扫描目录… 已发现 N 个程序」）
        let _ = app.emit("scan-progress", serde_json::json!({
            "phase": "walking",
            "found": walk_paths.len(),
            "done": 0,
            "total": 0,
        }));
    }

    let total = walk_paths.len();
    let mut done = 0usize;

        // 第二阶段：逐条解析名称与相对路径（图标提取移到导入阶段，扫描只收集清单，保证速度）
        for abs_path in &walk_paths {
            let path = Path::new(abs_path);
            let name = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("未知")
                .to_string();

            let (suggested_mode, rel_path) = if let Some(rel) = make_relative_path(&exe_dir, path) {
                ("relative", rel)
            } else {
                ("absolute", String::new())
            };

            results.push(ExeInfo {
                name: name.clone(),
                icon_path: String::new(),
                suggested_path_mode: suggested_mode.to_string(),
                relative_path: rel_path,
                absolute_path: abs_path.clone(),
                file_exists: true,
                steam: None,
                multi_exe: false,
                is_primary: true,
            });

            done += 1;
            let _ = app.emit("scan-progress", serde_json::json!({
                "phase": "walking",
                "found": total,
                "done": done,
                "total": total,
                "current": name,
            }));
        }

    Ok(ScanResult { results, total })
}

/// 扫描 Steam 安装目录：解析 steamapps/appmanifest_*.acf，为每个游戏生成一个
/// ExeInfo（name 用游戏名，absolute_path 用找到的游戏主程序，steam 字段带 appid/
/// installdir/steam_dir）。扫描阶段不提取图标、不下载封面（保证扫描速度），
/// 多候选 exe 的游戏逐个展开为多条（multi_exe=true，第一个 is_primary=true）。
/// 逐游戏推送 scan-progress 事件（phase="icons"，含 done/total 与当前游戏名）。
fn scan_steam_games(app: &tauri::AppHandle, steam_dir: &Path) -> Result<ScanResult, String> {
    let steamapps = steam_dir.join("steamapps");
    let games = read_steam_manifests(&steamapps);
    let exe_dir = get_exe_directory()?;
    let mut results = Vec::new();

    let total = games.len();
    let mut done = 0usize;

    for (appid, name, installdir) in games {
        let install_dir = steamapps.join("common").join(&installdir);
        // 找出全部候选 exe：多个时逐个展开为结果条目（前端红色提示用户勾选），
        // 第一个为主程序（默认选中）；图标提取与封面下载移到导入阶段
        let all_exes = find_game_exes(&install_dir);

        for (exe_idx, game_exe) in all_exes.iter().enumerate() {
            let is_primary = exe_idx == 0;
            let multi_exe = all_exes.len() > 1;
            // 多 exe 时名称附 exe 名区分（“游戏名 [exe名]”），单 exe 保持游戏名
            let display_name = if multi_exe {
                let exe_stem = Path::new(game_exe)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                format!("{} [{}]", name, exe_stem)
            } else {
                name.clone()
            };

            let (suggested_mode, rel_path) = if !game_exe.is_empty() {
                if let Some(rel) = make_relative_path(&exe_dir, Path::new(game_exe)) {
                    ("relative", rel)
                } else {
                    ("absolute", String::new())
                }
            } else {
                ("absolute", String::new())
            };

            results.push(ExeInfo {
                name: display_name,
                icon_path: String::new(),
                suggested_path_mode: suggested_mode.to_string(),
                relative_path: rel_path,
                absolute_path: game_exe.clone(),
                file_exists: !game_exe.is_empty(),
                steam: Some(SteamGameInfo {
                    app_id: appid.clone(),
                    install_dir: installdir.clone(),
                    exe_path: game_exe.clone(),
                    cover_path: String::new(),
                    steam_dir: steam_dir.to_string_lossy().to_string(),
                }),
                multi_exe,
                is_primary,
            });
        }

        done += 1;
        let _ = app.emit("scan-progress", serde_json::json!({
            "phase": "icons",
            "found": total,
            "done": done,
            "total": total,
            "current": name,
        }));
    }

    Ok(ScanResult { results, total })
}

// ─── 扫描结果导入（多线程） ───

/// 批量导入扫描结果的整体统计（后端命令返回值）
#[derive(Debug, Clone, Serialize)]
struct ImportScanSummary {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
    /// 新导入并已写盘的 Entry（含 id，前端直接同步内存状态）
    pub entries: Vec<Entry>,
}

/// 批量导入扫描结果：多线程提取图标（程序/Steam 通用）、下载 Steam 封面，
/// 构造 Entry 后统一追加写盘一次（避免前端逐条串行）。
/// 线程模型与 batch_refresh_icons 一致（std::thread，线程数 = min(条目数, 8)）。
/// 逐条推送 import-progress 事件（payload：done/total/name/status/message）。
#[command]
fn import_scan_items(
    app: tauri::AppHandle,
    items: Vec<ExeInfo>,
    category_id: String,
    current_env_id: String,
) -> Result<ImportScanSummary, String> {
    use std::sync::{Arc, Mutex};
    use std::sync::atomic::{AtomicUsize, Ordering};

    let total = items.len();
    if total == 0 {
        return Ok(ImportScanSummary { total: 0, success: 0, failed: 0, entries: Vec::new() });
    }

    let indexed: Vec<(usize, ExeInfo)> = items.into_iter().enumerate().collect();
    let results: Arc<Mutex<Vec<(usize, Entry)>>> = Arc::new(Mutex::new(Vec::with_capacity(total)));
    let failed_flags: Arc<Mutex<Vec<(usize, String)>>> = Arc::new(Mutex::new(Vec::new()));
    let done_count = Arc::new(AtomicUsize::new(0));

    // 线程数：条目数与 8 取小（COM/Shell 图标提取与 CDN 下载较重，过多线程收益递减）
    let thread_num = total.min(8).max(1);
    let chunk_size = (total + thread_num - 1) / thread_num;

    let mut handles = Vec::new();
    for chunk in indexed.chunks(chunk_size) {
        let results = Arc::clone(&results);
        let failed_flags = Arc::clone(&failed_flags);
        let done_count = Arc::clone(&done_count);
        let category_id = category_id.clone();
        let env_id = current_env_id.clone();
        let app = app.clone();
        let chunk_vec = chunk.to_vec();
        handles.push(std::thread::spawn(move || {
            for (idx, item) in chunk_vec {
                // COM 初始化：extract_icon_image 内部每次调用 com_init()，
                // 各工作线程独立初始化，无需额外处理
                let steam = item.steam.clone();
                let exe_path = steam.as_ref()
                    .map(|s| s.exe_path.clone())
                    .unwrap_or_else(|| item.absolute_path.clone());

                // 图标提取（失败不影响导入，使用默认图标）
                let icon_path = if exe_path.is_empty() {
                    String::new()
                } else {
                    extract_icon_to_png(&exe_path).unwrap_or_default()
                };

                // Steam 封面（本地缓存优先，缺失走 CDN；已有目标文件直接复用）
                let cover_path = if let Some(s) = &steam {
                    if s.steam_dir.is_empty() {
                        String::new()
                    } else {
                        get_steam_cover(Path::new(&s.steam_dir), &s.app_id)
                    }
                } else {
                    String::new()
                };

                let now = get_now_iso();
                let mut failed_msg = String::new();

                let entry = if let Some(s) = &steam {
                    if exe_path.is_empty() {
                        failed_msg = "游戏 exe 路径为空".to_string();
                    }
                    Entry {
                        id: String::new(),
                        name: item.name.clone(),
                        entry_type: "steam".to_string(),
                        category_id: category_id.clone(),
                        relative_path: String::new(),
                        absolute_paths: if exe_path.is_empty() {
                            HashMap::new()
                        } else {
                            let mut m = HashMap::new();
                            m.insert(env_id.clone(), exe_path.clone());
                            m
                        },
                        path_mode: "absolute".to_string(),
                        launch_args: String::new(),
                        working_directory: String::new(),
                        window_style: "normal".to_string(),
                        run_as_admin: false,
                        url: format!("steam://rungameid/{}", s.app_id),
                        icon: if icon_path.is_empty() {
                            ImageResource { res_type: "default".to_string(), source: String::new() }
                        } else {
                            ImageResource { res_type: "extracted".to_string(), source: icon_path.clone() }
                        },
                        cover: if cover_path.is_empty() {
                            CoverResource { enabled: false, source: String::new() }
                        } else {
                            CoverResource { enabled: true, source: cover_path }
                        },
                        tags: Vec::new(),
                        notes: String::new(),
                        last_used: String::new(),
                        add_time: now,
                    }
                } else {
                    if item.absolute_path.is_empty() {
                        failed_msg = "程序路径为空".to_string();
                    }
                    Entry {
                        id: String::new(),
                        name: item.name.clone(),
                        entry_type: "program".to_string(),
                        category_id: category_id.clone(),
                        relative_path: item.relative_path.clone(),
                        absolute_paths: if item.suggested_path_mode == "absolute" && !item.absolute_path.is_empty() {
                            let mut m = HashMap::new();
                            m.insert(env_id.clone(), item.absolute_path.clone());
                            m
                        } else {
                            HashMap::new()
                        },
                        path_mode: item.suggested_path_mode.clone(),
                        launch_args: String::new(),
                        working_directory: String::new(),
                        window_style: "normal".to_string(),
                        run_as_admin: false,
                        url: String::new(),
                        icon: if icon_path.is_empty() {
                            ImageResource { res_type: "default".to_string(), source: String::new() }
                        } else {
                            ImageResource { res_type: "extracted".to_string(), source: icon_path.clone() }
                        },
                        cover: CoverResource { enabled: false, source: String::new() },
                        tags: Vec::new(),
                        notes: String::new(),
                        last_used: String::new(),
                        add_time: now,
                    }
                };

                let status = if failed_msg.is_empty() { "success" } else { "failed" };
                let message = if !failed_msg.is_empty() {
                    failed_msg.clone()
                } else if icon_path.is_empty() && !exe_path.is_empty() {
                    "图标提取失败，使用默认图标".to_string()
                } else {
                    String::new()
                };

                if failed_msg.is_empty() {
                    let mut list = results.lock().unwrap();
                    list.push((idx, entry));
                } else {
                    let mut list = failed_flags.lock().unwrap();
                    list.push((idx, failed_msg));
                }

                let done = done_count.fetch_add(1, Ordering::SeqCst) + 1;
                // 逐条推送进度（前端导入进度弹窗实时展示）
                let _ = app.emit("import-progress", serde_json::json!({
                    "done": done,
                    "total": total,
                    "name": item.name,
                    "status": status,
                    "message": message,
                }));
            }
        }));
    }
    for h in handles {
        let _ = h.join();
    }

    // 汇总：按输入顺序排列，统一分配 id 后追加写盘一次
    let mut new_entries = Arc::try_unwrap(results)
        .map_err(|_| "结果收集失败".to_string())?
        .into_inner()
        .unwrap();
    new_entries.sort_by_key(|(idx, _)| *idx);
    let mut new_entries: Vec<Entry> = new_entries.into_iter().map(|(_, e)| e).collect();

    let data_dir = get_data_dir()?;
    let entries_path = data_dir.join("entries.json");
    let mut existing: Vec<Entry> = read_json_file(&entries_path).unwrap_or_default();
    let mut next_id = existing.iter()
        .filter_map(|e| e.id.strip_prefix("entry_").and_then(|s| s.parse::<u32>().ok()))
        .max()
        .unwrap_or(0) + 1;
    for e in &mut new_entries {
        e.id = format!("entry_{}", next_id);
        next_id += 1;
    }
    let success = new_entries.len();
    existing.append(&mut new_entries);
    write_json_file(&entries_path, &existing)?;

    let failed = total - success;
    Ok(ImportScanSummary { total, success, failed, entries: existing[existing.len() - success..].to_vec() })
}

// ─── 路径模式转换 ───

/// 将条目的路径在相对路径和绝对路径之间转换
/// - relative → absolute: 将 relative_path 解析为绝对路径，存入当前环境的 absolute_paths
/// - absolute → relative: 同盘符时用 ..\..\ 形式转为相对路径，不同盘符时失败
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertPathResult {
    pub success: bool,
    pub message: String,
    pub new_path_mode: String,
    pub new_relative_path: String,
    pub new_absolute_path: String,
}

#[command]
fn convert_path_mode(entry: Entry, current_env_id: String, target_mode: String) -> Result<ConvertPathResult, String> {
    let exe_dir = get_exe_directory()?;

    // 特殊 URI 条目（shell:/ms-settings:）不参与路径模式转换
    let cur_abs = entry.absolute_paths.get(&current_env_id).cloned().unwrap_or_default();
    if is_special_uri(&entry.relative_path) || is_special_uri(&cur_abs) {
        return Ok(ConvertPathResult {
            success: false,
            message: "系统功能/APPX 条目无需转换路径".to_string(),
            new_path_mode: entry.path_mode.clone(),
            new_relative_path: entry.relative_path.clone(),
            new_absolute_path: cur_abs,
        });
    }

    if target_mode == "relative" {
        // absolute → relative：取当前环境的绝对路径，转为相对路径
        let abs_path = match entry.absolute_paths.get(&current_env_id)
            .filter(|p| !p.is_empty()) {
            Some(p) => p,
            None => return Err("当前环境没有绝对路径，无法转换".to_string()),
        };

        let abs_path_obj = Path::new(abs_path);

        // 检查文件是否存在
        if !abs_path_obj.exists() {
            return Ok(ConvertPathResult {
                success: false,
                message: "绝对路径对应的文件不存在，无法转换".to_string(),
                new_path_mode: entry.path_mode.clone(),
                new_relative_path: entry.relative_path.clone(),
                new_absolute_path: entry.absolute_paths.get(&current_env_id).cloned().unwrap_or_default(),
            });
        }

        // 同盘符时计算相对路径
        if let Some(rel) = make_relative_path(&exe_dir, abs_path_obj) {
            return Ok(ConvertPathResult {
                success: true,
                message: "已转换为相对路径".to_string(),
                new_path_mode: "relative".to_string(),
                new_relative_path: rel,
                new_absolute_path: String::new(),
            });
        } else {
            return Ok(ConvertPathResult {
                success: false,
                message: "文件不在同盘符内，无法转为相对路径".to_string(),
                new_path_mode: entry.path_mode.clone(),
                new_relative_path: entry.relative_path.clone(),
                new_absolute_path: entry.absolute_paths.get(&current_env_id).cloned().unwrap_or_default(),
            });
        }
    } else if target_mode == "absolute" {
        // relative → absolute：将 relative_path 拼接 exe_dir 得到绝对路径
        if entry.relative_path.is_empty() {
            return Ok(ConvertPathResult {
                success: false,
                message: "没有相对路径，无法转换".to_string(),
                new_path_mode: entry.path_mode.clone(),
                new_relative_path: entry.relative_path.clone(),
                new_absolute_path: entry.absolute_paths.get(&current_env_id).cloned().unwrap_or_default(),
            });
        }

        let full = exe_dir.join(&entry.relative_path);
        let exists = full.exists();
        // 词法规范化消除 ..\ 段——路径条目里存的应是规范绝对路径，
        // 否则后续图标提取等 shell API 会拒绝（0x80070057）
        let full = normalize_path(&full);

        return Ok(ConvertPathResult {
            success: true,
            message: if exists {
                "已转换为绝对路径".to_string()
            } else {
                "已转换为绝对路径（文件当前不存在）".to_string()
            },
            new_path_mode: "absolute".to_string(),
            new_relative_path: entry.relative_path.clone(),
            new_absolute_path: full.to_string_lossy().to_string(),
        });
    } else {
        Err(format!("不支持的目标模式: {}", target_mode))
    }
}

// ─── 文件操作 ───

#[command]
fn copy_file_to_assets(source_path: String, target_subdir: String) -> Result<String, String> {
    let src = Path::new(&source_path);
    if !src.exists() {
        return Err(format!("源文件不存在: {}", source_path));
    }

    let assets_dir = get_assets_dir()?;
    let target_dir = assets_dir.join(&target_subdir);
    ensure_dir(&target_dir)?;

    // 生成目标文件名
    let ext = src.extension().and_then(|s| s.to_str()).unwrap_or("png");
    let stem = src.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
    let safe_stem: String = stem.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect();
    let timestamp = chrono::Local::now().format("%H%M%S%3f").to_string();
    let target_name = format!("{}_{}.{}", safe_stem, timestamp, ext);
    let target_path = target_dir.join(&target_name);

    fs::copy(&source_path, &target_path)
        .map_err(|e| format!("复制文件失败: {}", e))?;

    Ok(rel_to_exe_dir(&target_path))
}

// ─── Favicon 获取 ───

/// domain → 安全文件名（非法字符替换为 _，用于 assets/icons/custom/ 下的缓存命名）
fn safe_domain_filename(domain: &str) -> String {
    domain.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect()
}

/// favicon 多源获取（内部实现，供命令与图标重提取共用）。
/// 图标按 domain 命名缓存于 assets/icons/custom/{domain}.png：
/// 同 domain 的不同链接共用同一文件，缓存命中时不再重复下载。
/// force = true 时绕过缓存强制重新下载（重置图标语义：确保拿到最新图）。
/// 缓存未命中时按顺序尝试获取源（可在设置中自定义，模板支持占位符
/// {url} 完整链接 | {domain} 域名 | {scheme} 协议），任一步成功即返回
/// 保存后的相对路径；读取配置失败或列表为空时回落内置默认三源。
/// 每步 2 秒超时；全部失败返回 Err。
fn fetch_favicon_inner(url: &str, force: bool) -> Result<String, String> {
    let parsed = url::Url::parse(url)
        .map_err(|e| format!("URL 解析失败: {}", e))?;
    let domain = parsed.host_str()
        .ok_or("无法获取域名")?
        .to_string();

    // 同 domain 缓存命中：直接复用，不重复下载（重置场景 force 绕过）
    let cached_path = get_assets_dir()?
        .join("icons")
        .join("custom")
        .join(format!("{}.png", safe_domain_filename(&domain)));
    if !force && cached_path.exists() {
        return Ok(rel_to_exe_dir(&cached_path));
    }

    // 2 秒超时的客户端（总超时：连接 + 响应 + 读取响应体）
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    // 获取源模板：优先读用户设置（每条调用读一次小文件，开销可接受）；
    // 读失败或过滤空行后为空 → 回落内置默认
    let mut templates = get_data_dir()
        .ok()
        .and_then(|dir| read_json_file::<AppConfig>(&dir.join("config.json")).ok())
        .map(|cfg| cfg.favicon_api_sources)
        .unwrap_or_default();
    templates.retain(|t| !t.trim().is_empty());
    if templates.is_empty() {
        templates = default_favicon_api_sources();
    }

    // 模板替换占位符 → 实际请求地址，按序尝试
    let scheme = parsed.scheme();
    let candidates: Vec<String> = templates
        .iter()
        .map(|t| t.replace("{url}", url).replace("{domain}", &domain).replace("{scheme}", scheme))
        .collect();

    let mut last_err = String::new();
    for src in &candidates {
        match client.get(src).send() {
            Ok(resp) if resp.status().is_success() => match resp.bytes() {
                Ok(bytes) => {
                    if bytes.is_empty() {
                        last_err = format!("{} 返回空内容", src);
                        continue;
                    }
                    // 校验像不像图片（魔数）：png/jpg/gif/ico/bmp/webp
                    let b = &bytes;
                    let looks_like_image = b.starts_with(&[0x89, b'P', b'N', b'G'])
                        || b.starts_with(&[0xFF, 0xD8, 0xFF])
                        || b.starts_with(b"GIF8")
                        || (b.len() >= 4 && b[0] == 0 && b[1] == 0 && b[2] == 1 && b[3] == 0)
                        || (b.len() >= 2 && b[0] == 0x42 && b[1] == 0x4D)
                        || (b.len() >= 4 && &b[0..4] == b"RIFF" && b.len() >= 12 && &b[8..12] == b"WEBP");
                    if !looks_like_image {
                        last_err = format!("{} 返回的不是图片", src);
                        continue;
                    }
                    return save_favicon_bytes(&bytes, &domain);
                }
                Err(e) => last_err = format!("读取响应失败: {}", e),
            },
            Ok(resp) => last_err = format!("{} 状态码 {}", src, resp.status()),
            Err(e) => last_err = format!("请求失败: {}", e),
        }
    }
    Err(format!("无法获取 favicon（{}）", last_err))
}

/// 把 favicon 字节保存到 assets/icons/custom/{domain}.png，返回相对 exe 目录的路径。
/// 按 domain 命名：同 domain 不同链接共用同一文件（多线程并发写同一目标时容忍最后覆盖，
/// 内容同为该 domain 的合法 favicon，最终状态一致）。
fn save_favicon_bytes(bytes: &[u8], domain: &str) -> Result<String, String> {
    let assets_dir = get_assets_dir()?;
    let icon_dir = assets_dir.join("icons").join("custom");
    ensure_dir(&icon_dir)?;

    let file_path = icon_dir.join(format!("{}.png", safe_domain_filename(domain)));

    // 尝试用 image crate 解码并转为 PNG（统一格式）
    match image::load_from_memory(bytes) {
        Ok(img) => {
            img.to_rgba8().save(&file_path)
                .map_err(|e| format!("保存图标失败: {}", e))?;
        }
        Err(_) => {
            // 解码失败，直接保存原始字节
            fs::write(&file_path, bytes)
                .map_err(|e| format!("保存图标失败: {}", e))?;
        }
    }

    Ok(rel_to_exe_dir(&file_path))
}

#[command]
fn fetch_favicon(url: String, force: Option<bool>) -> Result<String, String> {
    fetch_favicon_inner(&url, force.unwrap_or(false))
}

// ─── 浏览器书签解析与导入 ───

/// 从 HTML 属性列表字符串中提取指定属性值。
/// 属性值可能被双引号包裹，也可能裸露（Netscape 格式导出的 HREF 总是带引号）。
/// 返回解码了常见 HTML 实体（&amp; &lt; &gt; &quot; &#39;）后的字符串。
fn extract_attr(attrs: &str, key: &str) -> Option<String> {
    // 找到属性名（前面必须是空白或行首，后面跟 =）
    let bytes = attrs.as_bytes();
    let pattern = format!("{}=", key);
    let mut search_from = 0usize;
    while let Some(rel) = attrs[search_from..].find(&pattern) {
        let start = search_from + rel;
        // 检查前一个字符是否为空白或属性行首
        let prev_ok = start == 0 || bytes[start - 1].is_ascii_whitespace() || bytes[start - 1] == b'"';
        if !prev_ok {
            search_from = start + pattern.len();
            continue;
        }
        let mut pos = start + pattern.len();
        // 跳过空白
        while pos < bytes.len() && bytes[pos].is_ascii_whitespace() { pos += 1; }
        if pos >= bytes.len() {
            return None;
        }
        if bytes[pos] == b'"' {
            // 带引号的值
            let value_start = pos + 1;
            let value_end = attrs[value_start..].find('"').map(|i| value_start + i)?;
            let raw = &attrs[value_start..value_end];
            return Some(decode_html_entities(raw));
        } else if bytes[pos] == b'\'' {
            let value_start = pos + 1;
            let value_end = attrs[value_start..].find('\'').map(|i| value_start + i)?;
            let raw = &attrs[value_start..value_end];
            return Some(decode_html_entities(raw));
        } else {
            // 裸值：读到下一个空白
            let value_start = pos;
            let value_end = attrs[value_start..]
                .find(|c: char| c.is_whitespace())
                .map(|i| value_start + i)
                .unwrap_or(attrs.len());
            let raw = &attrs[value_start..value_end];
            return Some(decode_html_entities(raw));
        }
    }
    None
}

/// 解码常见 HTML 实体
fn decode_html_entities(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

/// 将 base64 data URI（如 "data:image/png;base64,iVBOR..."）解码保存为图标文件。
/// 返回相对 exe 目录的路径（如 "assets/icons/bookmarks/icon_1400000000_0.png"）。
fn save_bookmark_icon(data_uri: &str, seq: usize) -> Result<String, String> {
    let base64_part = match data_uri.find("base64,") {
        Some(idx) => &data_uri[idx + 7..],
        None => return Err("不是 base64 data URI".to_string()),
    };
    if base64_part.is_empty() {
        return Err("data URI 内容为空".to_string());
    }

    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(base64_part.trim())
        .map_err(|e| format!("base64 解码失败: {}", e))?;

    if bytes.is_empty() || bytes.len() < 8 {
        return Err("图标数据过小".to_string());
    }

    // 通过魔数判断扩展名
    let ext = if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        "png"
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "jpg"
    } else if bytes.len() >= 4 && bytes[0] == 0 && bytes[1] == 0 && bytes[2] == 1 && bytes[3] == 0 {
        "ico" // ICO 魔数 00 00 01 00
    } else if bytes.starts_with(b"GIF8") {
        "gif"
    } else {
        // 未知格式，尝试用 image crate 解码为 PNG 统一处理
        match image::load_from_memory(&bytes) {
            Ok(img) => {
                let assets_dir = get_assets_dir()?;
                let icon_dir = assets_dir.join("icons").join("bookmarks");
                ensure_dir(&icon_dir)?;
                let ts = chrono::Local::now().format("%H%M%S%3f").to_string();
                let file_path = icon_dir.join(format!("icon_{}_{}.png", ts, seq));
                img.to_rgba8().save(&file_path)
                    .map_err(|e| format!("保存图标失败: {}", e))?;
                return Ok(rel_to_exe_dir(&file_path));
            }
            Err(_) => return Err("无法识别的图标格式".to_string()),
        }
    };

    // ico 格式转 PNG 保存（image crate 支持 ico 解码）
    if ext == "ico" {
        match image::load_from_memory(&bytes) {
            Ok(img) => {
                let assets_dir = get_assets_dir()?;
                let icon_dir = assets_dir.join("icons").join("bookmarks");
                ensure_dir(&icon_dir)?;
                let ts = chrono::Local::now().format("%H%M%S%3f").to_string();
                let file_path = icon_dir.join(format!("icon_{}_{}.png", ts, seq));
                img.to_rgba8().save(&file_path)
                    .map_err(|e| format!("保存图标失败: {}", e))?;
                return Ok(rel_to_exe_dir(&file_path));
            }
            Err(_) => return Err("ico 解码失败".to_string()),
        }
    }

    // png/jpg/gif 直接保存
    let assets_dir = get_assets_dir()?;
    let icon_dir = assets_dir.join("icons").join("bookmarks");
    ensure_dir(&icon_dir)?;
    let ts = chrono::Local::now().format("%H%M%S%3f").to_string();
    let file_path = icon_dir.join(format!("icon_{}_{}.{}", ts, seq, ext));
    fs::write(&file_path, &bytes)
        .map_err(|e| format!("写入图标文件失败: {}", e))?;

    Ok(rel_to_exe_dir(&file_path))
}

/// 解析 <DL> 内部（不含首尾标签）的一段内容，收集其中直接出现的链接与子文件夹。
/// links 与 folders 均为该层级的直接成员。
fn parse_bookmark_dl(content: &str, links: &mut Vec<BookmarkLink>, folders: &mut Vec<BookmarkFolder>, counters: &mut (usize, usize)) {
    let mut i = 0usize;
    let len = content.len();

    while i < len {
        // 寻找下一个 <DT>
        let Some(dt_rel) = content[i..].find("<DT>") else { break };
        let item_start = i + dt_rel + 4;

        // 判断是 <H3（文件夹）、<A（链接）还是 <DL（匿名内容块）
        let rest = &content[item_start..];
        let trimmed = rest.trim_start();
        let trim_off = rest.len() - trimmed.len();

        if trimmed.starts_with("<H3") {
            // ── 文件夹：解析标题与内容块，直属链接/子文件夹存入自身 ──
            let Some(tag_end_rel) = trimmed.find('>') else { break };
            let after_tag_off = trim_off + tag_end_rel + 1;
            let after_tag = &content[item_start + after_tag_off..];
            let Some(title_end_rel) = after_tag.find("</H3>") else { break };
            let title = decode_html_entities(after_tag[..title_end_rel].trim());

            let mut sub_folders = Vec::new();
            let mut sub_links = Vec::new();
            let consumed = if let Some(dl_rel) = after_tag[title_end_rel + 5..].find("<DL>") {
                // 有内容块：跳过开标签，然后找配对的 </DL>（处理嵌套）
                let inner = &after_tag[title_end_rel + 5 + dl_rel + 4..];
                match find_dl_end(inner) {
                    Some(end) => {
                        parse_bookmark_dl(&inner[..end], &mut sub_links, &mut sub_folders, counters);
                        // 内容块全部长度 = 5(</H3>) + dl_rel + 4(<DL>) + end + 5(</DL>)
                        title_end_rel + 5 + dl_rel + 4 + end + 5
                    }
                    None => {
                        // 未找到配对 </DL>：整段作为该文件夹内容，防解析重复
                        parse_bookmark_dl(inner, &mut sub_links, &mut sub_folders, counters);
                        usize::MAX
                    }
                }
            } else {
                // 没有内容块，空文件夹
                0
            };
            if consumed == usize::MAX {
                break;
            }
            folders.push(BookmarkFolder {
                title,
                links: sub_links,
                children: sub_folders,
            });
            counters.1 += 1;
            i = item_start + after_tag_off + consumed;
        } else if trimmed.starts_with("<A") {
            // ── 链接 ──
            let Some(tag_end_rel) = trimmed.find('>') else { break };
            let after_tag_off = trim_off + tag_end_rel + 1;
            let after_tag = &content[item_start + after_tag_off..];
            let Some(title_end_rel) = after_tag.find("</A>") else { break };
            let title = decode_html_entities(after_tag[..title_end_rel].trim());

            let a_attrs = &trimmed[..tag_end_rel];
            let href = extract_attr(a_attrs, "HREF").unwrap_or_default();
            let icon_uri = extract_attr(a_attrs, "ICON").unwrap_or_default();

            if !href.is_empty() && !href.starts_with("javascript:") && !href.starts_with("place:") {
                let icon_path = if icon_uri.starts_with("data:") {
                    match save_bookmark_icon(&icon_uri, counters.0) {
                        Ok(p) => p,
                        Err(_) => String::new(),
                    }
                } else {
                    String::new()
                };
                links.push(BookmarkLink {
                    title: if title.is_empty() { href.clone() } else { title },
                    url: href,
                    icon_path,
                });
                counters.0 += 1;
            }
            i = item_start + after_tag_off + title_end_rel + 4;
        } else if trimmed.starts_with("<DL>") {
            // ── 匿名 <DL>：内容并入当前层级 ──
            match find_dl_end(&trimmed[4..]) {
                Some(end) => {
                    parse_bookmark_dl(&trimmed[4..4 + end], links, folders, counters);
                    i = item_start + trim_off + 4 + end + 5;
                }
                None => {
                    parse_bookmark_dl(&trimmed[4..], links, folders, counters);
                    i = len;
                }
            }
        } else {
            // 未知标签（<p> 等）：跳过到下一个 <DT>
            let next_dt = rest[trim_off..].find("<DT>").map(|r| r + trim_off).unwrap_or(usize::MAX);
            let next_dl = rest[trim_off..].find("<DL>").map(|r| r + trim_off).unwrap_or(usize::MAX);
            let skip = next_dt.min(next_dl);
            if skip == usize::MAX {
                i = len;
            } else if skip == 0 {
                i += 1; // 防止死循环
            } else {
                i = item_start + skip;
            }
        }
    }
}

/// 在 inner 内查找配对的 </DL> 结束位置（inner 为 <DL> 之后的内容）。
/// 返回该 DL 块内部内容的长度（不含 </DL> 本身），找不到则返回 None。
fn find_dl_end(inner: &str) -> Option<usize> {
    let mut depth = 1usize;
    let mut scan = 0usize;
    while scan < inner.len() {
        // 取当前位置后的第一个开标签与闭标签，按位置先后处理
        let open = inner[scan..].find("<DL>").map(|r| scan + r);
        let close = inner[scan..].find("</DL>").map(|r| scan + r);
        match (open, close) {
            (Some(o), Some(c)) if c < o => {
                depth -= 1;
                scan = c + 5;
                if depth == 0 {
                    return Some(c);
                }
            }
            (Some(o), _) => {
                depth += 1;
                scan = o + 4;
            }
            (None, Some(c)) => {
                depth -= 1;
                scan = c + 5;
                if depth == 0 {
                    return Some(c);
                }
            }
            (None, None) => break,
        }
    }
    None
}

/// 解析 Netscape 书签文件（Chrome/Edge/360 等浏览器导出的 HTML）
#[command]
fn parse_bookmarks(file_path: String) -> Result<BookmarkParseResult, String> {
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("读取书签文件失败: {}", e))?;

    // 校验格式
    let upper = content.to_uppercase();
    if !upper.contains("NETSCAPE-Bookmark-file") && !upper.contains("<DL>") {
        return Err("不是有效的浏览器书签文件（Netscape Bookmark 格式）".to_string());
    }

    // 提取最外层 <DL>...</DL>（从第一个 <DL> 到最后一个 </DL>）
    let Some(first_dl) = content.find("<DL>") else {
        return Err("书签文件中没有 <DL> 结构".to_string());
    };
    let Some(last_dl_end) = content.rfind("</DL>") else {
        return Err("书签文件中没有 </DL> 结束标签".to_string());
    };
    let outer = &content[first_dl + 4..last_dl_end];

    let mut root_links: Vec<BookmarkLink> = Vec::new();
    let mut folders: Vec<BookmarkFolder> = Vec::new();
    let mut counters = (0usize, 0usize); // (链接计数, 文件夹计数)

    parse_bookmark_dl(outer, &mut root_links, &mut folders, &mut counters);

    if folders.is_empty() && root_links.is_empty() {
        return Err("书签文件中没有解析到任何书签内容".to_string());
    }

    // 统计总数（递归）
    fn count_folder(f: &BookmarkFolder) -> (usize, usize) {
        let mut links = f.links.len();
        let mut folders = 1;
        for c in &f.children {
            let (l, fo) = count_folder(c);
            links += l;
            folders += fo;
        }
        (links, folders)
    }
    let mut total_links = root_links.len();
    let mut total_folders = 0;
    for f in &folders {
        let (l, fo) = count_folder(f);
        total_links += l;
        total_folders += fo;
    }

    Ok(BookmarkParseResult {
        folders,
        root_links,
        total_links,
        total_folders,
    })
}

// ─── 过期资源清理 ───

/// 清理扫描/清理结果
#[derive(Debug, Clone, Serialize)]
pub struct AssetCleanResult {
    /// 过期文件数
    pub orphan_count: usize,
    /// 过期文件总字节数
    pub orphan_bytes: u64,
    /// 实际删除的文件数（仅 clean_assets 有值）
    pub deleted_count: usize,
    /// 实际释放的字节数（仅 clean_assets 有值）
    pub deleted_bytes: u64,
}

/// 收集所有 JSON 数据中对 assets 资源的引用（相对路径集合）。
/// 覆盖四处引用：条目图标、条目封面、分类图标、LOGO 图标/图片。
/// 注意：封面即使 enabled=false，只要 source 非空也算被引用（编辑器只有点"清除"才清空 source）。
///
/// 健壮性设计（防止误删）：
/// 1. 任何 JSON 读取/解析失败立即报错中止——引用收集不全时清理会误删在用文件；
/// 2. 兜底文件名匹配：除完整路径匹配外，同时记录引用文件名，扫描时"路径或文件名任一命中即算被引用"，
///    兼容旧格式/异常格式路径（如反斜杠、缺少 assets/ 前缀）。
fn collect_asset_refs() -> Result<(std::collections::HashSet<String>, std::collections::HashSet<String>), String> {
    let data_dir = get_data_dir()?;
    let mut refs = std::collections::HashSet::new();
    let mut ref_names = std::collections::HashSet::new();

    let push = |source: &str, refs: &mut std::collections::HashSet<String>, ref_names: &mut std::collections::HashSet<String>| {
        let s = source.trim();
        if s.is_empty() {
            return;
        }
        // 统一转正斜杠后记录完整路径
        let norm = s.replace('\\', "/");
        refs.insert(norm);
        // 兜底：同时记录纯文件名（取最后一段）
        if let Some(name) = s.trim_end_matches(['\\', '/']).rsplit(['\\', '/']).next() {
            if !name.is_empty() {
                ref_names.insert(name.to_string());
            }
        }
    };

    let entries: Vec<Entry> = read_json_file(&data_dir.join("entries.json"))
        .map_err(|e| format!("引用收集中止（防止误删）：{}", e))?;
    for e in &entries {
        push(&e.icon.source, &mut refs, &mut ref_names);
        push(&e.cover.source, &mut refs, &mut ref_names);
    }

    let categories: Vec<Category> = read_json_file(&data_dir.join("categories.json"))
        .map_err(|e| format!("引用收集中止（防止误删）：{}", e))?;
    for c in &categories {
        push(&c.icon.source, &mut refs, &mut ref_names);
    }

    let config: AppConfig = read_json_file(&data_dir.join("config.json"))
        .map_err(|e| format!("引用收集中止（防止误删）：{}", e))?;
    if config.logo_icon_type == "custom" {
        push(&config.logo_icon, &mut refs, &mut ref_names);
    }
    push(&config.logo_image, &mut refs, &mut ref_names);

    Ok((refs, ref_names))
}

/// 扫描清理范围内未被引用的过期文件（白名单 6 个子目录，不碰 assets 根部的前端资源）。
/// 返回 (过期文件列表, 过期总字节数)。
///
/// 安全防线：
/// 1. 完整路径匹配（规范化正斜杠）；
/// 2. 文件名兜底匹配——任一 JSON 中出现过同名文件即视为被引用；
/// 3. 防呆拒清：引用集合为空但磁盘上有文件时，视为数据异常，拒绝清理并报错。
fn find_orphan_assets() -> Result<(Vec<std::path::PathBuf>, u64), String> {
    let assets_dir = get_assets_dir()?;
    let (refs, ref_names) = collect_asset_refs()?;

    // 清理范围白名单（相对 assets 的子目录）
    let scopes = [
        "icons/extracted",
        "icons/custom",
        "icons/bookmarks",
        "icons/file_cache",
        "covers/custom",
        "covers/steam",
        "logo",
    ];

    // 先数一遍磁盘文件总数，用于防呆判断
    let mut disk_file_count = 0usize;
    for sub in &scopes {
        let dir = assets_dir.join(sub);
        if !dir.is_dir() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&dir) {
            disk_file_count += entries.flatten().filter(|i| i.path().is_file()).count();
        }
    }

    // 防呆拒清：引用为空但磁盘有文件，说明 JSON 数据异常（解析失败已提前报错，
    // 这里兜底空文件等极端情况），宁可拒绝清理也不能全删
    if refs.is_empty() && ref_names.is_empty() && disk_file_count > 0 {
        return Err(format!(
            "引用数据为空但磁盘上存在 {} 个资源文件，疑似数据异常，已拒绝清理以防误删",
            disk_file_count
        ));
    }

    let mut orphans = Vec::new();
    let mut total_bytes = 0u64;

    for sub in &scopes {
        let dir = assets_dir.join(sub);
        if !dir.is_dir() {
            continue;
        }
        let entries = std::fs::read_dir(&dir).map_err(|e| format!("读取目录失败 {}: {}", sub, e))?;
        for item in entries.flatten() {
            let path = item.path();
            if !path.is_file() {
                continue;
            }
            // 转成 assets/xxx/yyy.png 形式与引用集合比对
            let rel = path.strip_prefix(&assets_dir).unwrap_or(&path);
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            // 双重匹配：完整路径命中 或 文件名兜底命中，均视为被引用
            let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            let is_referenced = refs.contains(&rel_str) || (!file_name.is_empty() && ref_names.contains(file_name));
            if !is_referenced {
                if let Ok(meta) = item.metadata() {
                    total_bytes += meta.len();
                }
                orphans.push(path);
            }
        }
    }

    Ok((orphans, total_bytes))
}

/// 扫描过期资源（无副作用，仅统计）
#[command]
fn scan_orphan_assets() -> Result<AssetCleanResult, String> {
    let (orphans, bytes) = find_orphan_assets()?;
    Ok(AssetCleanResult {
        orphan_count: orphans.len(),
        orphan_bytes: bytes,
        deleted_count: 0,
        deleted_bytes: 0,
    })
}

/// 将文件移入回收站（SHFileOperationW + FOF_ALLOWUNDO）。
/// paths 为绝对路径列表，返回失败文件描述列表（空表示全部成功）。
/// 注意：pFrom 必须是双 \0 结尾的宽字符串，多个路径用 \0 分隔。
#[cfg(windows)]
fn recycle_files(paths: &[std::path::PathBuf]) -> Vec<String> {
    use std::os::windows::ffi::OsStrExt;

    if paths.is_empty() {
        return Vec::new();
    }

    // 构造双 \0 结尾的多路径宽字符串（\0 分隔）
    let mut from: Vec<u16> = Vec::new();
    for p in paths {
        for c in p.as_os_str().encode_wide() {
            from.push(c);
        }
        from.push(0);
    }
    from.push(0);

    let mut op = SHFILEOPSTRUCTW {
        hwnd: HWND::default(),
        wFunc: 3, // FO_DELETE
        pFrom: PCWSTR(from.as_ptr()),
        pTo: PCWSTR::null(),
        // FOF_ALLOWUNDO(64) 进回收站 | FOF_NOCONFIRMATION(16) 不弹确认 | FOF_SILENT(4) 不显示进度
        fFlags: (64 | 16 | 4) as u16,
        fAnyOperationsAborted: BOOL::default(),
        hNameMappings: std::ptr::null_mut(),
        lpszProgressTitle: PCWSTR::null(),
    };

    let code = unsafe { SHFileOperationW(&mut op) };
    if code != 0 {
        // 整体失败：列出所有路径供用户排查
        return paths.iter()
            .map(|p| format!("{}: SHFileOperation 错误码 {}", p.display(), code))
            .collect();
    }
    // 部分被用户中止也算失败
    if op.fAnyOperationsAborted.as_bool() {
        return vec!["操作被中止".to_string()];
    }
    Vec::new()
}

#[cfg(not(windows))]
fn recycle_files(_paths: &[std::path::PathBuf]) -> Vec<String> {
    vec!["仅支持 Windows 平台".to_string()]
}

/// 清理过期资源（重新扫描后移入回收站，返回实际清理结果）
#[command]
fn clean_orphan_assets() -> Result<AssetCleanResult, String> {
    let (orphans, bytes) = find_orphan_assets()?;

    // 统计总大小（回收站删除前统计）
    let mut deleted_bytes = 0u64;
    for path in &orphans {
        if let Ok(meta) = std::fs::metadata(path) {
            deleted_bytes += meta.len();
        }
    }

    let failures = recycle_files(&orphans);
    Ok(AssetCleanResult {
        orphan_count: orphans.len(),
        orphan_bytes: bytes,
        deleted_count: orphans.len() - failures.len(),
        deleted_bytes: if failures.is_empty() { deleted_bytes } else { 0 },
    })
}

/// 删除条目后的资源联动清理：
/// 前端删除条目并保存 JSON 后调用，传入被删条目的图标/封面相对路径列表。
/// 复用 collect_asset_refs 重建「删除后」的全量引用集合，候选文件只有
/// 「完整路径或文件名均不再被引用」才移入回收站（复用 recycle_files，可还原）。
/// domain 缓存图标（{domain}.png）被同 domain 其他链接引用时按文件名命中，不会误删。
/// 返回实际移入回收站的文件数（失败/路径不在 assets 内的不计入）。
#[command]
fn cleanup_deleted_assets(sources: Vec<String>) -> Result<usize, String> {
    // 过滤空路径
    let candidates: Vec<String> = sources
        .into_iter()
        .map(|s| s.trim().replace('\\', "/"))
        .filter(|s| !s.is_empty())
        .collect();
    if candidates.is_empty() {
        return Ok(0);
    }

    // 白名单前缀：只允许删 assets 下资源目录内的文件，防误传任意路径
    const ALLOWED_PREFIXES: [&str; 7] = [
        "assets/icons/extracted/",
        "assets/icons/custom/",
        "assets/icons/bookmarks/",
        "assets/icons/file_cache/",
        "assets/covers/custom/",
        "assets/covers/steam/",
        "assets/logo/",
    ];

    let assets_dir = get_assets_dir()?;

    // 收集「删除后」的最新引用集合（条目/分类/LOGO 全量 JSON）
    let (refs, ref_names) = collect_asset_refs()?;

    let mut to_recycle: Vec<std::path::PathBuf> = Vec::new();
    for src in &candidates {
        // 仅处理 assets 白名单前缀内的路径
        if !ALLOWED_PREFIXES.iter().any(|p| src.starts_with(p)) {
            continue;
        }
        // 拼绝对路径（词法规范化，防 .. 段）
        let abs = assets_dir.join(
            src.strip_prefix("assets/")
                .unwrap_or(src),
        );
        let abs = normalize_path(&abs);
        // 确认文件仍存在
        if !abs.is_file() {
            continue;
        }
        // 引用判断：完整路径或文件名任一命中即视为仍被引用，跳过
        let file_name = abs.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if refs.contains(src) || (!file_name.is_empty() && ref_names.contains(file_name)) {
            continue;
        }
        to_recycle.push(abs);
    }

    if to_recycle.is_empty() {
        return Ok(0);
    }

    let failures = recycle_files(&to_recycle);
    Ok(to_recycle.len() - failures.len())
}

// ════════════════════════════════════════════
// 开机自启（注册表 HKCU Run 键）
// ════════════════════════════════════════════

/// 自启命令串中的静默启动参数（开机自启且勾选"不显示主界面"时附加）
const AUTORUN_HIDDEN_ARG: &str = "--hidden";

/// 注册表 HKCU Run 键操作封装（仅 Windows）
#[cfg(windows)]
mod autorun_registry {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::System::Registry::*;

    pub const RUN_KEY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    pub const VALUE_NAME: &str = "MyLauncher";

    fn to_wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    /// 打开（或创建）HKCU Run 键
    /// 注：不用 RegCreateKeyExW（在 windows 0.58 中需 Win32_Security feature），RegCreateKeyW 足够
    fn open_run_key() -> Result<HKEY, String> {
        let sub_key_w = to_wide(RUN_KEY_PATH);
        unsafe {
            let mut key = HKEY::default();
            let result = RegCreateKeyW(
                HKEY_CURRENT_USER,
                PCWSTR(sub_key_w.as_ptr()),
                &mut key,
            );
            if result.is_err() {
                return Err(format!("打开注册表 Run 键失败: {:?}", result));
            }
            Ok(key)
        }
    }

    /// 写入 Run 值（REG_SZ，存在则覆盖）
    pub fn set_run_value(data: &str) -> Result<(), String> {
        let value_name = to_wide(VALUE_NAME);
        let value_data = to_wide(data);
        let bytes: Vec<u8> = value_data
            .iter()
            .flat_map(|w| w.to_le_bytes())
            .collect();
        let key = open_run_key()?;
        let result = unsafe {
            RegSetValueExW(
                key,
                PCWSTR(value_name.as_ptr()),
                0,
                REG_SZ,
                Some(&bytes),
            )
        };
        unsafe { let _ = RegCloseKey(key); }
        if result.is_err() {
            return Err(format!("写入注册表值失败: {:?}", result));
        }
        Ok(())
    }

    /// 读取 Run 值（值不存在或类型非 REG_SZ 返回 None）
    pub fn get_run_value() -> Option<String> {
        let value_name = to_wide(VALUE_NAME);
        let key = open_run_key().ok()?;
        let mut buf_len: u32 = 0;
        let query = unsafe {
            RegQueryValueExW(
                key,
                PCWSTR(value_name.as_ptr()),
                None,
                None,
                None,
                Some(&mut buf_len),
            )
        };
        if query.is_err() || buf_len == 0 {
            unsafe { let _ = RegCloseKey(key); }
            return None;
        }
        let mut buf = vec![0u8; buf_len as usize];
        let result = unsafe {
            RegQueryValueExW(
                key,
                PCWSTR(value_name.as_ptr()),
                None,
                None,
                Some(buf.as_mut_ptr()),
                Some(&mut buf_len),
            )
        };
        unsafe { let _ = RegCloseKey(key); }
        if result.is_err() {
            return None;
        }
        // 字节 → UTF-16 字符串（去掉结尾 NUL）
        let words: Vec<u16> = buf
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        let s = String::from_utf16_lossy(&words);
        let s = s.trim_end_matches('\0').to_string();
        if s.is_empty() { None } else { Some(s) }
    }

    /// 删除 Run 值（值不存在视为成功）
    pub fn delete_run_value() -> Result<(), String> {
        let value_name = to_wide(VALUE_NAME);
        let key = open_run_key()?;
        let result = unsafe { RegDeleteValueW(key, PCWSTR(value_name.as_ptr())) };
        unsafe { let _ = RegCloseKey(key); }
        // ERROR_FILE_NOT_FOUND(2) = 值本来就不存在，视为成功
        if result.is_err() && result.0 != 2 {
            return Err(format!("删除注册表值失败: {:?}", result));
        }
        Ok(())
    }
}

/// 当前 exe 绝对路径（用于自启注册表命令串）
fn current_exe_path() -> Result<String, String> {
    std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| format!("获取 exe 路径失败: {}", e))
}

/// 按开关与静默选项写/删自启注册表值；命令串格式: "exe路径" [--hidden]
fn apply_autorun_registry(enabled: bool, silent: bool) -> Result<(), String> {
    if !enabled {
        #[cfg(windows)]
        autorun_registry::delete_run_value()?;
        return Ok(());
    }
    let exe = current_exe_path()?;
    let data = if silent {
        format!("\"{}\" {}", exe, AUTORUN_HIDDEN_ARG)
    } else {
        format!("\"{}\"", exe)
    };
    #[cfg(windows)]
    autorun_registry::set_run_value(&data)?;
    #[cfg(not(windows))]
    let _ = data;
    Ok(())
}

/// 启动时校正自启注册表值：程序被移动后旧路径失效，用当前实际路径重写（保留 --hidden 标记）
fn fix_autorun_registry() {
    #[cfg(windows)]
    {
        if let Some(data) = autorun_registry::get_run_value() {
            if let Ok(exe) = current_exe_path() {
                let quoted = format!("\"{}\"", exe);
                if !data.starts_with(&quoted) {
                    let silent = data.contains(AUTORUN_HIDDEN_ARG);
                    let new_data = if silent {
                        format!("\"{}\" {}", exe, AUTORUN_HIDDEN_ARG)
                    } else {
                        quoted
                    };
                    let _ = autorun_registry::set_run_value(&new_data);
                }
            }
        }
    }
}

/// 读取自启注册表值当前状态（是否存在 + 是否静默启动）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutorunStatus {
    pub enabled: bool,
    pub silent: bool,
}

fn get_autorun_status() -> AutorunStatus {
    #[cfg(windows)]
    {
        if let Some(data) = autorun_registry::get_run_value() {
            return AutorunStatus {
                enabled: true,
                silent: data.contains(AUTORUN_HIDDEN_ARG),
            };
        }
    }
    AutorunStatus { enabled: false, silent: false }
}

/// 设置开机自启（enabled=false 时删除注册表值；silent 控制自启时是否隐藏主窗口）
#[command]
fn set_autorun(enabled: bool, silent: bool) -> Result<(), String> {
    apply_autorun_registry(enabled, silent)
}

/// 查询当前自启状态（读注册表实际值，非前端内存）
#[command]
fn get_autorun() -> AutorunStatus {
    get_autorun_status()
}

// ════════════════════════════════════════════
// 应用入口
// ════════════════════════════════════════════

/// 托盘菜单项 ID
const TRAY_MENU_SHOW: &str = "show";
const TRAY_MENU_QUIT: &str = "quit";

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            use tauri::{
                menu::{Menu, MenuItem},
                tray::{TrayIconBuilder, TrayIconEvent},
                Manager,
            };

            // ─── 系统托盘：图标 + 右键菜单 ───
            let show_item = MenuItem::with_id(app, TRAY_MENU_SHOW, "恢复窗口", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, TRAY_MENU_QUIT, "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        // 恢复窗口：显示 + 取消最小化 + 抢占前台
                        TRAY_MENU_SHOW => {
                            if let Some(win) = app.get_webview_window("main") {
                                let _ = win.show();
                                let _ = win.unminimize();
                                let _ = win.set_focus();
                            }
                        }
                        // 退出：清理托盘图标后真正退出
                        TRAY_MENU_QUIT => {
                            if let Some(tray) = app.tray_by_id("main-tray") {
                                let _ = tray.set_visible(false);
                            }
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|_tray, event| {
                    // 左键单击托盘图标也恢复窗口（最小化状态下常用）
                    if let TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, button_state: tauri::tray::MouseButtonState::Up, .. } = event {
                        if let Some(win) = _tray.app_handle().get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.unminimize();
                            let _ = win.set_focus();
                        }
                    }
                })
                .build(app)?;

            // ─── 启动显示策略 ───
            // 窗口在 tauri.conf.json 中默认不可见；手动双击启动（无 --hidden 参数）显示主窗口，
            // 开机自启静默模式（带 --hidden 参数）保持隐藏只留托盘
            if let Some(win) = app.get_webview_window("main") {
                let silent_start = std::env::args().any(|a| a == AUTORUN_HIDDEN_ARG);
                if !silent_start {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }

            // ─── 自启路径校正：程序移动位置后用当前路径重写注册表（保留静默标记）───
            fix_autorun_registry();

            Ok(())
        })
        .on_window_event(|window, event| {
            use tauri::Manager;
            // 主窗口关闭请求 → 按设置分流
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    // 读取关闭行为设置（每次关闭时读取，开销可接受）
                    let close_action = get_data_dir()
                        .ok()
                        .and_then(|dir| read_json_file::<AppConfig>(&dir.join("config.json")).ok())
                        .map(|cfg| cfg.close_action)
                        .unwrap_or_else(default_close_action);

                    if close_action == "close" {
                        // 直接退出：清理托盘图标后关闭窗口（触发默认退出流程）
                        if let Some(tray) = window.app_handle().tray_by_id("main-tray") {
                            let _ = tray.set_visible(false);
                        }
                        // 不调用 prevent_close，让窗口正常关闭退出
                    } else {
                        // 默认：隐藏到托盘
                        api.prevent_close();
                        let _ = window.hide();
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            init_data_dir,
            load_all_data,
            save_entries,
            save_categories,
            save_environments,
            save_config,
            get_exe_dir,
            get_now,
            resolve_path,
            check_paths_batch,
            launch_program,
            get_exe_info,
            get_file_info,
            resolve_lnk,
            scan_directory,
            scan_steam_app,
            import_scan_items,
            convert_path_mode,
            copy_file_to_assets,
            fetch_favicon,
            parse_bookmarks,
            list_appx_apps,
            extract_appx_icon,
            open_in_explorer,
            scan_orphan_assets,
            clean_orphan_assets,
            cleanup_deleted_assets,
            refresh_entry_icon,
            batch_refresh_icons,
            set_autorun,
            get_autorun
        ])
        .run(tauri::generate_context!())
        .expect("MyLauncher 启动失败");
}
