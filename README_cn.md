# RCargo - 支持重定向 target 目录的 Rust Cargo 包装器

**语言**: [English](README.md) | [中文](README_cn.md)

RCargo 是一个高性能的 Rust cargo 包装器，可以将目标目录重定向到快速存储（通常是 ramdisk），从而显著加快编译速度并保护您的固态硬盘。

## ✨ 特性

- 🚀 **项目独立的目标目录**: 每个项目都有自己的独立缓存目录
- 🔧 **环境变量支持**: 使用 `RCARGO_TARGET_DIR` 自定义缓存位置
- 📊 **缓存管理**: 内置命令查看和清理缓存
- 🔄 **Cargo 透明传递**: 所有标准 cargo 命令都能透明工作
- 💾 **节省空间**: 轻松清理未使用的缓存目录

## 🚀 快速开始

### 安装

```bash
# 从源码安装
git clone https://github.com/sakarie9/rcargo.git
cd rcargo
cargo install --path .

# 或直接从 git 安装
cargo install --git https://github.com/sakarie9/rcargo.git
```

### 基本用法

像使用 `cargo` 一样使用 `rcargo`，但您的目标文件将放在指定目录中，默认为 `/tmp/rcargo_targets`

设置 `RCARGO_TARGET_DIR` 环境变量来自定义目标目录。

### 💡 使用技巧

#### 使用 rcargo 替代 cargo

有两种便捷的方式可以让 `rcargo` 成为您默认的 Rust 构建工具：

##### 方法一：符号链接

配置 `rcargo` 在项目目录中创建名为 `target` 的符号链接。这样，`cargo` 和 `rcargo` 都将无缝使用相同的重定向目标目录：

```bash
export RCARGO_TARGET_LINK_NAME=target
```

> [!NOTE]
> 如果符号链接的目标目录不存在，`cargo` 命令将会失败。在使用符号链接方式之前，您必须至少运行一次 `rcargo` 来创建目标目录。

##### 方法二：Shell 别名

创建别名，当您输入 `cargo` 时自动使用 `rcargo`：

```bash
alias cargo=rcargo
```

将此行添加到您的 shell 配置文件（`.bashrc`、`.zshrc` 等）中以使其持久生效。

## 📋 命令

### 标准 Cargo 命令

所有标准 cargo 命令都能无缝工作：

```bash
rcargo build          # 构建项目
rcargo test           # 运行测试  
rcargo run            # 运行项目
rcargo check          # 检查错误
rcargo clean          # 清理目标目录（cargo 的 clean）
```

### RCargo 命令

#### 查看缓存大小

```bash
# 显示当前项目缓存大小
rcargo size

# 显示所有缓存的项目
rcargo size --all
```

**示例输出：**

```text
Current project 'my-app' target size: 125.4 MB
```

```text
All cached project target directories:
  my-app-a1b2c3d: 125.4 MB
  web-server-x7y8z9: 89.2 MB  
  cli-tool-m4n5o6: 45.8 MB
Total cache size: 260.4 MB
```

#### 清理缓存

```bash
# 清理当前项目缓存（需要确认）
rcargo purge

# 清理所有项目缓存（需要确认）
rcargo purge --all

# 跳过确认提示
rcargo purge --yes
rcargo purge --all --yes
```

#### 版本信息

```bash
# 显示 rcargo 和 cargo 的版本
rcargo --version
```

## ⚙️ 配置

### 环境变量

| 变量 | 描述 | 默认值 |
|------|------|--------|
| `RCARGO_CARGO_PATH` | 自定义 cargo 二进制位置 | `/usr/bin/cargo` |
| `RCARGO_TARGET_DIR` | 自定义目标目录位置 | `/tmp/rcargo_targets` |
| `RCARGO_NO_TARGET_LINK` | 不创建链接到指定 target 目录的链接 | `false` |
| `RCARGO_TARGET_LINK_NAME` | target 的目录链接名 | `target_rcargo` |

### 示例

```bash
# 使用自定义缓存目录
export RCARGO_TARGET_DIR="/mnt/ramdisk/cargo_cache"
rcargo build

# 使用 SSD 作为缓存
export RCARGO_TARGET_DIR="/fast-ssd/cargo_cache" 
rcargo build
```

## 📁 缓存目录结构

RCargo 使用以下格式创建唯一的缓存目录：

```text
{项目名称}-{路径哈希}
```

其中：

- `项目名称`: 从 `Cargo.toml` 提取或使用目录名
- `路径哈希`: 项目路径的 7 位 MD5 哈希值

**示例：**

```text
/tmp/rcargo_targets/
├── my-web-app-a1b2c3d/     # /home/user/projects/my-web-app
├── my-web-app-x7y8z9a/     # /home/user/work/my-web-app (不同路径)
└── cli-tool-m4n5o6p/       # /home/user/tools/cli-tool
```

这确保了同名但位置不同的项目拥有独立的缓存。
