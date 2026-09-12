@echo off
chcp 65001 >nul
title MyLauncher Dev Server
cd /d "%~dp0"

echo ============================================
echo   MyLauncher 开发环境启动器
echo   - 首次启动请耐心等待依赖安装与 Rust 编译
echo   - 关闭本窗口即停止开发服务器
echo ============================================
echo.

if not exist node_modules (
  echo [dev] 未检测到 node_modules，正在安装依赖...
  call npm install
  if errorlevel 1 (
    echo [dev] 依赖安装失败，请检查 npm 与网络。
    pause
    exit /b 1
  )
)

echo [dev] 启动 Tauri 开发环境（前端 HMR + Rust 后端）...
call npm run tauri dev

echo.
echo [dev] 开发服务器已退出。
pause