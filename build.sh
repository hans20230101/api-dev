#!/bin/bash
# 设置临时目录环境变量（解决 SQLite 编译权限问题）
export TMP="E:\ai-test\api-dev\.build-tmp"
export TEMP="E:\ai-test\api-dev\.build-tmp"
export TMPDIR="E:\ai-test\api-dev\.build-tmp"

# 确保临时目录存在
mkdir -p "$TMP"

# 运行打包
cargo tauri build
