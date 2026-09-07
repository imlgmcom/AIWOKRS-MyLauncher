# MyLauncher 项目分析文档

> 本文档从一个"只懂 HTML + JS + CSS 的普通人"视角出发，把整个项目拆解清楚。你可以直接把本文档丢给 AI，让它照着做一个一模一样的项目。

---

## 一、项目是做什么的

**MyLauncher** 是一个 **Windows 桌面端的绿色便携程序管理器**。

简单说就是：你电脑上有很多软件、游戏、网址、文件夹、文件，散落在各个磁盘和目录里，找起来很麻烦。MyLauncher 就是帮你把这些东西 **统一收录到一个界面里**，按分类整理好，一键就能启动——类似一个"个人版的开始菜单"。

### 核心功能一览

| 功能 | 说明 |
|------|------|
| **收录程序** | 手动添加 exe，或者把 exe/快捷方式直接拖进窗口，自动识别名称、图标、路径 |
| **收录网址** | 添加网址收藏，自动抓取网站 favicon 当图标 |
| **收录文件夹/文件** | 添加任意文件夹或文件，自动提取系统图标 |
| **收录系统功能** | 添加 Windows 系统功能（任务管理器、设备管理器等），用 emoji 当图标 |
| **收录 APPX/UWP 应用** | 枚举 Windows 开始菜单的 UWP 应用并添加 |
| **分类管理** | 支持多级分类树（父分类→子分类），每个分类可自定义图标 |
| **多视图模式** | 每个分类可切换 4 种视图：图标网格、竖向卡片、横向卡片、表格 |
| **多环境** | 支持多个"环境"（比如公司电脑和家里电脑），同一个程序在不同环境可以有不同的绝对路径 |
| **路径检测** | 自动检测收录的程序是否还存在，路径失效会标红提醒 |
| **相对/绝对路径** | 同一磁盘内的程序自动用相对路径（方便 U 盘携带），跨磁盘用绝对路径 |
| **浏览器书签导入** | 解析 Chrome/Edge 导出的书签 HTML 文件，批量导入网址收藏 |
| **批量操作** | 批量选择条目 → 批量删除、批量移动分类、批量转换路径模式、批量重新提取图标 |
| **文件夹扫描** | 指定一个文件夹，自动扫描里面所有 exe 一键批量添加 |
| **系统托盘** | 关闭窗口时隐藏到托盘，双击托盘图标恢复 |
| **开机自启** | 支持注册表自启动，可选静默模式（开机不弹窗只留托盘） |
| **明暗主题** | 支持明亮/暗黑/跟随系统三种主题 |
| **过期资源清理** | 删除条目后自动清理不再被引用的图标/封面文件（移入回收站，可还原） |

---

## 二、项目技术栈

### 整体架构

```
┌─────────────────────────────────────────────────┐
│                   桌面窗口                        │
│  ┌───────────────────────────────────────────┐  │
│  │           WebView2 (浏览器内核)             │  │
│  │  ┌─────────────────────────────────────┐  │  │
│  │  │     前端：Vue 3 + TypeScript        │  │  │
│  │  │     构建工具：Vite                   │  │  │
│  │  │     UI：纯 CSS（Fluent Design 风格）  │  │  │
│  │  └─────────────────────────────────────┘  │  │
│  └──────────────┬────────────────────────────┘  │
│                 │ invoke() 调用                   │
│  ┌──────────────┴────────────────────────────┐  │
│  │           后端：Rust (Tauri v2)            │  │
│  │  - 文件读写（JSON 持久化）                  │  │
│  │  - Windows API 调用（图标提取、程序启动）    │  │
│  │  - 网络请求（favicon 下载）                │  │
│  │  - 系统托盘、注册表操作                     │  │
│  └───────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
```

### 技术栈清单

| 层面 | 技术 | 版本 | 用途 |
|------|------|------|------|
| **桌面框架** | Tauri | v2 | 把网页打包成 Windows 桌面应用，提供前后端通信桥梁 |
| **前端框架** | Vue 3 | ^3.5.13 | 组件化 UI 开发，使用 `<script setup>` 语法 |
| **类型系统** | TypeScript | ~5.6.2 | 给 JS 加类型约束 |
| **构建工具** | Vite | ^6.0.3 | 前端开发服务器和打包 |
| **后端语言** | Rust | 2021 edition | 编写桌面端原生功能 |
| **样式方案** | 纯 CSS | - | CSS 变量实现主题切换，Fluent Design 风格 |
| **状态管理** | Vue 3 reactive | 内置 | 用 Vue 自带的响应式 API 做全局状态管理（没有用 Pinia/Vuex） |
| **Tauri 插件** | @tauri-apps/plugin-dialog | ^2.7.2 | 文件选择/保存对话框 |

### Rust 后端依赖

| 库 | 用途 |
|---|------|
| `tauri` | 桌面框架核心（含 `protocol-asset` 资源协议 + `tray-icon` 托盘图标） |
| `serde` + `serde_json` | JSON 序列化/反序列化（数据持久化） |
| `walkdir` | 递归遍历文件夹（扫描 exe） |
| `base64` | base64 解码（书签图标 data URI） |
| `image` | 图片处理（图标提取后转 RGBA → 保存 PNG） |
| `reqwest` | HTTP 请求（下载网站 favicon） |
| `url` | URL 解析（提取域名） |
| `chrono` | 时间处理（时间戳、文件命名） |
| `encoding_rs` | 字符编码（GBK 解码兜底） |
| `windows` | Windows API 绑定（Shell 图标提取、程序启动、注册表、回收站等） |

---

## 三、项目目录结构

```
MyLauncher-src/
├── index.html                    # HTML 入口，标题"MyLauncher - 绿色便携程序管理器"
├── package.json                  # 前端依赖和脚本
├── vite.config.ts                # Vite 配置（路径别名 @、Tauri 开发端口）
├── tsconfig.json                 # TypeScript 配置
│
├── public/                       # 静态资源（vite.svg、tauri.svg）
│
├── src/                          # 前端源码
│   ├── main.ts                   # 入口：创建 Vue 实例，挂载 #app
│   ├── App.vue                   # 根组件：整体布局、对话框管理、拖拽添加、右键菜单
│   │
│   ├── types/
│   │   └── index.ts              # 所有 TypeScript 类型定义（Entry、Category、Environment 等）
│   │
│   ├── api/
│   │   └── index.ts              # Tauri 命令封装层（前端调用后端功能的桥梁）
│   │
│   ├── store/
│   │   └── index.ts              # 全局状态管理（Vue reactive + CRUD 操作 + 主题管理）
│   │
│   ├── components/
│   │   ├── nav/
│   │   │   ├── TopNav.vue        # 顶部导航栏（Logo、搜索框、设置按钮）
│   │   │   ├── SideNav.vue       # 左侧分类导航（分类树、拖拽排序）
│   │   │   └── FooterNav.vue     # 底部工具栏（视图切换、添加条目、管理分类等）
│   │   │
│   │   ├── views/
│   │   │   ├── IconGridView.vue   # 图标网格视图
│   │   │   ├── VerticalCardView.vue  # 竖向卡片视图
│   │   │   ├── HorizontalCardView.vue # 横向卡片视图
│   │   │   └── TableView.vue      # 表格视图
│   │   │
│   │   ├── dialogs/
│   │   │   ├── EntryEditor.vue      # 条目编辑对话框（添加/编辑程序、网址等）
│   │   │   ├── CategoryManager.vue  # 分类管理对话框
│   │   │   ├── CategoryEditPanel.vue # 分类编辑面板
│   │   │   ├── EnvironmentManager.vue # 环境管理对话框
│   │   │   ├── ScanDialog.vue       # 文件夹扫描对话框
│   │   │   ├── BookmarkImportDialog.vue # 书签导入对话框
│   │   │   ├── DeleteCategoryDialog.vue # 删除分类确认对话框
│   │   │   ├── SettingsDialog.vue   # 设置对话框
│   │   │   ├── AboutDialog.vue      # 关于对话框
│   │   │   ├── SystemAppPicker.vue  # 系统功能选择器
│   │   │   └── BatchIconProgressDialog.vue # 批量图标提取进度
│   │   │
│   │   ├── ContextMenu.vue        # 右键菜单组件
│   │   ├── CategoryIcon.vue       # 分类图标组件
│   │   ├── IconPicker.vue         # 图标选择器（emoji / 自定义图片）
│   │   ├── PathModeFields.vue     # 路径模式字段组件
│   │   └── BatchToolbar.vue       # 批量操作工具栏
│   │
│   ├── composables/               # Vue 组合式函数
│   │   ├── useToast.ts            # Toast 提示
│   │   ├── useDismissable.ts      # 点击外部关闭
│   │   ├── useEntryView.ts        # 条目视图逻辑
│   │   └── useAssetImagePicker.ts # 资源图片选择
│   │
│   ├── data/
│   │   ├── emojiLibrary.ts        # Emoji 表情库数据
│   │   └── systemTools.ts         # 系统功能列表数据
│   │
│   ├── utils/
│   │   └── toggleInSet.ts         # Set 集合切换工具函数
│   │
│   └── styles/
│       ├── global.css             # 全局样式（CSS 变量、主题、通用组件样式）
│       └── form.css               # 表单样式
│
└── src-tauri/                     # Rust 后端
    ├── Cargo.toml                 # Rust 依赖配置
    ├── tauri.conf.json            # Tauri 应用配置（窗口尺寸、打包设置等）
    ├── src/
    │   ├── main.rs                # Rust 入口（调用 lib.rs 的 run()）
    │   └── lib.rs                 # 后端核心逻辑（约 2970 行）
    │                              # - 数据持久化（JSON 读写）
    │                              # - 路径解析（相对/绝对路径互转）
    │                              # - 程序启动（ShellExecuteW）
    │                              # - 图标提取（IShellItemImageFactory）
    │                              # - 快捷方式解析（IShellLinkW）
    │                              # - APPX 应用枚举（PowerShell Get-StartApps）
    │                              # - Favicon 下载（reqwest 多源获取）
    │                              # - 书签解析（Netscape HTML 格式）
    │                              # - 文件夹扫描（walkdir 递归）
    │                              # - 过期资源清理（引用检测 + 回收站）
    │                              # - 系统托盘 + 开机自启（注册表）
    └── icons/                     # 应用图标文件
```

---

## 四、数据模型

所有数据以 JSON 文件形式保存在程序同目录的 `data/` 文件夹下：

```
data/
├── config.json         # 应用配置（Logo、主题、关闭行为等）
├── categories.json     # 分类列表
├── entries.json        # 所有条目（程序/网址/文件夹等）
└── environments.json   # 环境列表
```

资源文件（图标、封面）保存在 `assets/` 文件夹：

```
assets/
├── icons/
│   ├── extracted/      # 自动提取的 exe/文件图标
│   ├── custom/         # 自定义图标 + favicon 缓存（按域名命名）
│   ├── bookmarks/      # 书签导入的图标
│   └── file_cache/     # 文件夹/文件类型图标缓存
├── covers/
│   └── custom/         # 自定义封面图片
└── logo/               # Logo 图片
```

### 核心数据结构

**条目（Entry）** - 一条收录的程序/网址/文件夹：
```json
{
  "id": "entry_1",
  "name": "VS Code",
  "type": "program",           // program | url | folder | file | system | appx
  "category_id": "cat_001",
  "relative_path": "..\\..\\Tools\\VSCode\\Code.exe",
  "absolute_paths": { "env_001": "D:\\Tools\\VSCode\\Code.exe" },
  "path_mode": "relative",     // relative | absolute
  "launch_args": "",
  "working_directory": "",
  "window_style": "normal",    // normal | maximized | minimized
  "run_as_admin": false,
  "url": "",
  "icon": { "type": "extracted", "source": "assets/icons/extracted/Code_143025001.png" },
  "cover": { "enabled": false, "source": "" },
  "tags": [],
  "notes": "",
  "last_used": "2026-08-11T10:30:00",
  "add_time": "2026-08-01T09:00:00"
}
```

**分类（Category）**：
```json
{
  "id": "cat_001",
  "name": "开发工具",
  "parent_id": null,           // null = 顶级分类
  "icon": { "type": "builtin", "source": "dev" },
  "view_mode": "icon_grid",    // icon_grid | vertical_card | horizontal_card | table
  "sort_order": 1,
  "is_top_level": true
}
```

**环境（Environment）**：
```json
{
  "id": "env_001",
  "name": "本机",
  "description": "当前电脑",
  "drive_mapping": {},
  "is_current": true
}
```

---

## 五、前后端通信机制

前端通过 `@tauri-apps/api/core` 的 `invoke()` 函数调用 Rust 后端的命令，Rust 端用 `#[command]` 宏注册命令。

**前端调用示例**（`api/index.ts`）：
```typescript
// 启动程序
export async function launch_program(entry: Entry, currentEnvId: string): Promise<boolean> {
  return invoke<boolean>('launch_program', { entry, currentEnvId })
}

// 提取 exe 图标
export async function get_exe_info(exePath: string): Promise<ExeInfo> {
  return invoke<ExeInfo>('get_exe_info', { exePath })
}
```

**Rust 端注册**（`lib.rs`）：
```rust
#[command]
fn launch_program(entry: Entry, current_env_id: String) -> Result<bool, String> {
    // ... 启动程序的逻辑
}

#[command]
fn get_exe_info(exe_path: String) -> Result<ExeInfo, String> {
    // ... 提取图标的逻辑
}
```

### 后端提供的全部命令

| 命令 | 功能 |
|------|------|
| `init_data_dir` | 初始化数据和资源目录 |
| `load_all_data` | 加载全部 JSON 数据 |
| `save_entries/categories/environments/config` | 保存各类数据 |
| `get_exe_dir` | 获取 exe 所在目录 |
| `get_now` | 获取当前时间字符串 |
| `resolve_path` | 解析条目路径（相对→绝对） |
| `check_paths_batch` | 批量检查路径是否存在 |
| `convert_path_mode` | 路径模式互转 |
| `launch_program` | 启动程序/打开网址 |
| `open_in_explorer` | 在资源管理器中定位文件 |
| `get_exe_info` | 获取 exe 信息 + 提取图标 |
| `get_file_info` | 获取文件/文件夹信息 + 提取图标 |
| `resolve_lnk` | 解析 .lnk 快捷方式 |
| `scan_directory` | 扫描文件夹中的 exe |
| `copy_file_to_assets` | 复制文件到资源目录 |
| `fetch_favicon` | 下载网站 favicon |
| `parse_bookmarks` | 解析浏览器书签 HTML |
| `list_appx_apps` | 枚举 UWP 应用 |
| `extract_appx_icon` | 提取 UWP 应用图标 |
| `refresh_entry_icon` | 重新提取单条图标 |
| `batch_refresh_icons` | 批量重新提取图标（多线程） |
| `scan_orphan_assets` | 扫描过期资源 |
| `clean_orphan_assets` | 清理过期资源（移入回收站） |
| `cleanup_deleted_assets` | 删除条目后联动清理资源 |
| `set_autorun` | 设置开机自启 |
| `get_autorun` | 查询自启状态 |

---

## 六、UI 界面布局

```
┌──────────────────────────────────────────────────────┐
│  [Logo] MyLauncher              [🔍 搜索...]  [⚙]   │  ← TopNav 顶部导航
├──────────┬───────────────────────────────────────────┤
│          │                                           │
│  开发工具  │    ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐     │
│  游戏     │    │ 📦  │ │ 📦  │ │ 📦  │ │ 📦  │     │  ← 内容区
│  影音     │    │ VS  │ │ Py  │ │ Git │ │ npm │     │    （动态视图）
│  系统工具  │    │Code │ │charm│ │     │ │     │     │
│  网址收藏  │    └─────┘ └─────┘ └─────┘ └─────┘     │
│          │                                           │
│  ← SideNav │   ┌─────┐ ┌─────┐                      │
│   侧导航   │    │ 📦  │ │ 📦  │                      │
│          │    │ nod │ │ den │                      │
│          │    └─────┘ └─────┘                      │
├──────────┴───────────────────────────────────────────┤
│  [+ 添加] [分类管理] [环境管理] [扫描] [📋书签] [⚙]  │  ← FooterNav 底部工具栏
│                                    [▦] [☰] [▤] [≡]  │  ← 视图切换按钮
└──────────────────────────────────────────────────────┘
```

### 4 种视图模式

1. **icon_grid** - 图标网格：大图标 + 名称，类似 Windows 桌面
2. **vertical_card** - 竖向卡片：上方大封面图 + 下方名称
3. **horizontal_card** - 横向卡片：左侧封面 + 右侧信息
4. **table** - 表格：多列信息展示

### 主题系统

用 CSS 变量实现，`html` 标签上加 `dark` 类切换到暗色模式：
- `:root` 定义明亮主题变量（主色 `#0078d4`，白底黑字）
- `html.dark` 覆盖为暗色变量（主色 `#4cc2ff`，深灰底白字）
- 字体使用 `Segoe UI` + `Microsoft YaHei UI`（Windows 原生风格）

---

## 七、关键交互流程

### 1. 添加程序（拖拽）

```
用户拖入 exe 文件
  → Tauri onDragDropEvent 触发
  → 判断文件类型（exe/lnk/其他）
  → 调用后端 get_exe_info / resolve_lnk / get_file_info
  → 后端提取图标保存到 assets/icons/extracted/
  → 自动计算相对路径（同盘）或绝对路径（跨盘）
  → 前端创建条目草稿 → 保存到 entries.json
```

### 2. 启动程序

```
用户双击条目
  → 前端调用 launch_program(entry, currentEnvId)
  → 后端 resolve_path 解析出实际路径
  → 检查文件是否存在
  → ShellExecuteW 启动程序（支持管理员权限、窗口模式、启动参数）
  → 更新 last_used 时间
```

### 3. 多环境路径管理

```
同一个程序在不同电脑上路径不同：
  - 相对路径（relative_path）：所有环境通用，如 "..\..\Tools\app.exe"
  - 绝对路径（absolute_paths）：按环境 ID 存储，如 { "env_001": "D:\Tools\app.exe" }
  - path_mode 决定优先用哪种路径

切换环境时，只显示当前环境有路径的条目（相对路径始终显示）
```

---

## 八、如何告诉 AI 做一个一样的项目

以下是你可以直接复制给 AI 的需求描述：

---

> **帮我做一个 Windows 桌面应用，叫"MyLauncher"，是一个绿色便携程序管理器。具体要求如下：**
>
> ### 技术选型
> - 桌面框架用 **Tauri v2**
> - 前端用 **Vue 3 + TypeScript + Vite**，组件用 `<script setup>` 语法
> - 样式用 **纯 CSS**（不用任何 UI 框架），风格参考 Windows 11 的 **Fluent Design**
> - 全局状态管理用 Vue 3 的 `reactive`（不用 Pinia/Vuex）
> - 后端用 **Rust**，通过 Tauri 的 `invoke` 命令与前端通信
>
> ### 数据存储
> - 所有数据以 **JSON 文件**保存在程序同目录的 `data/` 文件夹下（config.json、categories.json、entries.json、environments.json）
> - 图标和封面图片保存在 `assets/` 文件夹下的子目录中
> - 不需要数据库
>
> ### 核心功能
>
> **1. 条目管理（CRUD）**
> - 支持 6 种条目类型：程序（exe）、网址（url）、文件夹（folder）、文件（file）、系统功能（system）、APPX/UWP 应用（appx）
> - 每个条目有：名称、类型、所属分类、路径（相对+绝对）、启动参数、工作目录、窗口模式（正常/最大化/最小化）、管理员运行、图标、封面图、标签、备注、最后使用时间、添加时间
> - 支持拖拽 exe/lnk 文件到窗口自动添加（自动识别名称和图标）
>
> **2. 分类管理**
> - 多级分类树（父分类→子分类→孙分类...）
> - 每个分类有：名称、图标（emoji 或自定义图片）、视图模式、排序
> - 分类支持拖拽排序和移动层级
> - 删除分类时可选：条目变未分类 / 连同条目一起删除 / 移动到其他分类
>
> **3. 多环境支持**
> - 可以创建多个"环境"（如公司电脑、家里电脑）
> - 同一条目在不同环境可以有不同的绝对路径
> - 相对路径在所有环境通用
> - 支持路径模式互转（相对 ↔ 绝对），跨磁盘无法转相对路径
>
> **4. 图标系统**
> - 程序/文件/文件夹：通过 Windows Shell API 自动提取系统图标，保存为 PNG
> - 网址：多源 favicon 下载（a.favicon.im → faviconsnap → 原站 /favicon.ico），按域名缓存
> - 系统功能：使用 emoji 字符
> - APPX 应用：通过 shell:AppsFolder URI 提取
> - 支持自定义图标（上传图片）和 emoji 选择器
> - 支持批量重新提取图标（多线程 + 进度事件推送）
>
> **5. 视图系统**
> - 每个分类独立设置视图模式，4 种：图标网格、竖向卡片、横向卡片、表格
> - 底部工具栏切换视图
>
> **6. 批量操作**
> - 批量选择模式：勾选多个条目
> - 支持：批量删除、批量移动到其他分类、批量转换路径模式、批量重新提取图标
> - 全选、反选、清空选择
>
> **7. 其他功能**
> - 全局搜索（搜索名称/标签/备注/网址）
> - 右键菜单（打开、编辑、打开目录、删除）
> - 文件夹扫描（指定目录递归扫描 exe 批量添加）
> - 浏览器书签导入（解析 Netscape HTML 格式的书签文件）
> - 路径有效性检测（批量检查条目路径是否还存在）
> - 过期资源清理（删除条目后自动清理无人引用的图标/封面，移入回收站可还原）
> - 系统托盘（关闭窗口隐藏到托盘，双击恢复）
> - 开机自启（写注册表 HKCU Run 键，支持静默模式）
> - 明暗主题（亮/暗/跟随系统，CSS 变量切换）
>
> ### UI 布局
> - 顶部：Logo + 搜索框 + 设置按钮
> - 左侧：分类导航树（一级分类无子分类时隐藏侧栏）
> - 中间：内容区（动态视图组件）
> - 底部：工具栏（添加条目、分类管理、环境管理、扫描目录、书签导入、设置、视图切换）
> - 窗口默认 1200×800，最小 900×600
>
> ### 重要细节
> - 程序是**绿色便携**的：所有数据和资源都在 exe 同目录下，不写注册表（除自启功能外），不依赖安装，拷贝整个文件夹到另一台电脑就能用
> - 相对路径用 `..\..\` 形式（从 exe 目录出发），同盘符自动用相对路径
> - 图标提取后保存为 PNG 文件，用 Tauri 的 `asset://` 协议在前端显示
> - 图标更新后用 `?v=版本号` 查询参数绕过 WebView 缓存
> - 删除文件统一移入回收站（SHFileOperationW + FOF_ALLOWUNDO），不永久删除
> - 启动程序用 ShellExecuteW（支持管理员提权 UAC）

---

## 九、补充说明

### 为什么用 Tauri 而不是 Electron？
- Tauri 打包后体积小（约 5-10MB），Electron 动辄 100MB+
- Tauri 用系统自带的 WebView2（Windows 10/11 已内置），不需要捆绑 Chromium
- Tauri 后端用 Rust，性能好、内存占用低
- 对于"绿色便携"的定位来说，体积小是核心需求

### 为什么不用 UI 框架（Element UI / Ant Design 等）？
- 项目 UI 不复杂，纯 CSS + CSS 变量完全够用
- 减少依赖体积，保持便携性
- Fluent Design 风格的 CSS 变量已经封装在 `global.css` 中

### 项目的"绿色便携"设计
- 所有数据（JSON）和资源（图标/封面）都保存在 exe 同目录下
- 不写 AppData、不写注册表（除自启功能外）
- 同盘符内的程序自动用相对路径，整个文件夹拷贝到 U 盘换台电脑也能用
- 路径失效时会标红提醒（换了环境盘符不同）
