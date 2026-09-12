// ════════════════════════════════════════════
// 自动版本号：构建前把 tauri.conf.json 的 version 改写为
//   yy.M.d  如 26.9.12（无前导零，semver 合法）
// 由 package.json 的 tauri 脚本自动调用，无需手动改版本号。
// ════════════════════════════════════════════

import { readFileSync, writeFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'

const __dirname = dirname(fileURLToPath(import.meta.url))
const confPath = join(__dirname, '..', 'src-tauri', 'tauri.conf.json')

const now = new Date()
const yy = now.getFullYear() % 100
const M = now.getMonth() + 1
const d = now.getDate()
const version = `${yy}.${M}.${d}`

const conf = JSON.parse(readFileSync(confPath, 'utf-8'))
if (conf.version === version) {
  console.log(`[set-version] 版本号已是 ${version}，无需更新`)
} else {
  conf.version = version
  // 保留 2 空格缩进与结尾换行，与原文件风格一致
  writeFileSync(confPath, JSON.stringify(conf, null, 2) + '\n', 'utf-8')
  console.log(`[set-version] 版本号已更新为 ${version}`)
}
