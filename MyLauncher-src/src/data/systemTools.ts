// ════════════════════════════════════════════
// 系统功能静态清单（内置，选中后可自由改名/换图标/编辑）
// ════════════════════════════════════════════

/** 系统功能条目定义 */
export interface SystemTool {
  name: string
  /** emoji 图标 */
  emoji: string
  /** 启动目标：可执行文件名（如 taskmgr.exe）或 URI（ms-settings: / shell:） */
  target: string
  /** 启动参数（预留，一般为空） */
  args?: string
  /** 分组名 */
  group: string
  /** 简要说明 */
  desc: string
}

/** 内置系统功能清单 */
export const SYSTEM_TOOLS: SystemTool[] = [
  // ─── 系统管理 ───
  { name: '任务管理器', emoji: '📊', target: 'taskmgr.exe', group: '系统管理', desc: '查看进程与性能' },
  { name: '设备管理器', emoji: '🔧', target: 'devmgmt.msc', group: '系统管理', desc: '管理硬件与驱动' },
  { name: '磁盘管理', emoji: '💾', target: 'diskmgmt.msc', group: '系统管理', desc: '分区与磁盘操作' },
  { name: '事件查看器', emoji: '📜', target: 'eventvwr.msc', group: '系统管理', desc: '查看系统日志' },
  { name: '服务', emoji: '⚙️', target: 'services.msc', group: '系统管理', desc: '管理系统服务' },
  { name: '计算机管理', emoji: '🖥️', target: 'compmgmt.msc', group: '系统管理', desc: '综合管理工具' },
  { name: '注册表编辑器', emoji: '🗃️', target: 'regedit.exe', group: '系统管理', desc: '编辑系统注册表' },
  { name: '本地组策略', emoji: '📋', target: 'gpedit.msc', group: '系统管理', desc: '组策略编辑器（家庭版不可用）' },
  { name: '控制面板', emoji: '🎛️', target: 'control.exe', group: '系统管理', desc: '传统控制面板' },
  { name: '系统配置', emoji: '🧰', target: 'msconfig.exe', group: '系统管理', desc: '启动项与引导设置' },
  { name: '系统信息', emoji: 'ℹ️', target: 'msinfo32.exe', group: '系统管理', desc: '查看系统详细信息' },
  { name: '磁盘清理', emoji: '🧹', target: 'cleanmgr.exe', group: '系统管理', desc: '清理系统垃圾' },
  { name: '本地安全策略', emoji: '🔐', target: 'secpol.msc', group: '系统管理', desc: '安全策略（家庭版不可用）' },
  { name: '打印管理', emoji: '🖨️', target: 'printmanagement.msc', group: '系统管理', desc: '打印机与打印队列' },
  { name: '性能监视器', emoji: '📈', target: 'perfmon.msc', group: '系统管理', desc: '性能计数器监控' },
  { name: '资源监视器', emoji: '📡', target: 'resmon.exe', group: '系统管理', desc: 'CPU/内存/磁盘/网络实时监控' },
  { name: '字符映射表', emoji: '🔤', target: 'charmap.exe', group: '系统管理', desc: '查找并复制特殊字符' },
  { name: '步骤记录器', emoji: '📝', target: 'psr.exe', group: '系统管理', desc: '录制操作步骤截图' },

  // ─── Windows 设置 ───
  { name: '设置主页', emoji: '🪟', target: 'ms-settings:', group: 'Windows 设置', desc: '打开 Windows 设置' },
  { name: '显示设置', emoji: '🖥️', target: 'ms-settings:display', group: 'Windows 设置', desc: '分辨率与缩放' },
  { name: '声音设置', emoji: '🔊', target: 'ms-settings:sound', group: 'Windows 设置', desc: '音量与输出设备' },
  { name: '网络设置', emoji: '🌐', target: 'ms-settings:network', group: 'Windows 设置', desc: '网络与 Internet' },
  { name: '蓝牙设备', emoji: '🔵', target: 'ms-settings:bluetooth', group: 'Windows 设置', desc: '蓝牙与其他设备' },
  { name: '个性化', emoji: '🎨', target: 'ms-settings:personalization', group: 'Windows 设置', desc: '壁纸主题与颜色' },
  { name: '应用列表', emoji: '📦', target: 'ms-settings:appsfeatures', group: 'Windows 设置', desc: '已安装应用管理' },
  { name: '账户设置', emoji: '👤', target: 'ms-settings:yourinfo', group: 'Windows 设置', desc: '账户信息' },
  { name: 'Windows 更新', emoji: '🔄', target: 'ms-settings:windowsupdate', group: 'Windows 设置', desc: '系统更新' },
  { name: '存储设置', emoji: '🗄️', target: 'ms-settings:storagesense', group: 'Windows 设置', desc: '存储空间管理' },
  { name: '电源和电池', emoji: '🔋', target: 'ms-settings:powersleep', group: 'Windows 设置', desc: '电源与睡眠' },
  { name: '时间和语言', emoji: '🕒', target: 'ms-settings:dateandtime', group: 'Windows 设置', desc: '日期与时间' },
  { name: '辅助功能', emoji: '♿', target: 'ms-settings:easeofaccess', group: 'Windows 设置', desc: '辅助功能选项' },
  { name: '隐私和安全性', emoji: '🛡️', target: 'ms-settings:privacy', group: 'Windows 设置', desc: '隐私设置' },

  // ─── 常用工具 ───
  { name: '记事本', emoji: '📄', target: 'notepad.exe', group: '常用工具', desc: '文本编辑' },
  { name: '计算器', emoji: '🧮', target: 'calc.exe', group: '常用工具', desc: '计算器' },
  { name: '画图', emoji: '🖌️', target: 'mspaint.exe', group: '常用工具', desc: '图像编辑' },
  { name: '命令提示符', emoji: '⌨️', target: 'cmd.exe', group: '常用工具', desc: '命令行终端' },
  { name: 'PowerShell', emoji: '🔷', target: 'powershell.exe', group: '常用工具', desc: 'PowerShell 终端' },
  { name: '运行对话框', emoji: '🏃', target: 'ms-settings:', args: 'Run', group: '常用工具', desc: 'Win+R 运行' },
  { name: '截图工具', emoji: '✂️', target: 'snippingtool.exe', group: '常用工具', desc: '屏幕截图' },
  { name: '远程桌面', emoji: '🖥️', target: 'mstsc.exe', group: '常用工具', desc: '远程桌面连接' },
  { name: '磁盘碎片整理', emoji: '🧭', target: 'dfrgui.exe', group: '常用工具', desc: '优化驱动器' },
  { name: '任务计划程序', emoji: '⏰', target: 'taskschd.msc', group: '常用工具', desc: '定时任务管理' },
  { name: '凭据管理器', emoji: '🔑', target: 'ms-settings:credentials', group: '常用工具', desc: '保存的凭据' },
  { name: '环境变量', emoji: '🌍', target: 'ms-settings:about', group: '常用工具', desc: '系统信息页（可进入高级设置）' },
  { name: 'DirectX 诊断', emoji: '🎮', target: 'dxdiag.exe', group: '常用工具', desc: '显卡与声音诊断' },
]
