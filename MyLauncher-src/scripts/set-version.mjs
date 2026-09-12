// ════════════════════════════════════════════
// 自动版本号：构建前把 tauri.conf.json 的 version 改写为
//   1.{YY}{MM}{DD}.{HH}{mm}   如 1.260911.1030
// 由 package.json 的 tauri 脚本自动调用，无需手动改版本号。
// ════════════════════════════════════════════

import { readFileSync, writeFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'

const __dirname = dirname(fileURLToPath(import.meta.url))
const confPath = join(__dirname, '..', 'src-tauri', 'tauri.conf.json')

const now = new Date()
const pad = (n) => String(n).padStart(2, '0')
// HHMM 取整数值去前导零，避免 semver 数字段前导零报错（如 08:13 -> 813）
const hhmm = now.getHours() * 100 + now.getMinutes()
const version = `1.${pad(now.getFullYear() % 100)}${pad(now.getMonth() + 1)}${pad(now.getDate())}.${hhmm}`

const conf = JSON.parse(readFileSync(confPath, 'utf-8'))
if (conf.version === version) {
  console.log(`[set-version] 版本号已是 ${version}，无需更新`)
} else {
  conf.version = version
  // 保留 2 空格缩进与结尾换行，与原文件风格一致
  writeFileSync(confPath, JSON.stringify(conf, null, 2) + '\n', 'utf-8')
  console.log(`[set-version] 版本号已更新为 ${version}`)
}
