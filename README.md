# RRDP — Rust Remote Desktop Protocol

`rrdp` 是一个 [xfreerdp3](https://www.freerdp.com/) 的简洁命令行包装工具，用 Rust 编写。免去记忆繁琐的 FreeRDP 命令行参数，提供交互式连接管理。

## 功能

- **交互模式** — 运行 `rrdp` 或 `rrdp select` 进入 TUI 界面，↑↓ 选择连接，Enter 确认
- **连接管理** — 保存、加载、编辑、删除远程桌面连接配置
- **安全设计** — 密码仅输入时存在内存中，不写入配置文件
- **中文输出** — 所有提示和错误信息均为中文，错误码附带中文释义
- **智能参数** — 自动跳过无效参数（如空的域名），自动检测 `xfreerdp3` / `xfreerdp` 二进制

## 安装

### 前置条件

需要安装 FreeRDP：

```bash
sudo pacman -S freerdp
```

### 编译安装

```bash
git clone <你的仓库地址>
cd rrdp
cargo build --release
sudo cp target/release/rrdp /usr/local/bin/
```

或者直接运行（不安装）：

```bash
cargo run -- <子命令>
```

## 使用

### 交互模式（推荐）

```bash
# 直接运行进入交互菜单
rrdp

# 或使用 select 子命令
rrdp select
```

交互菜单支持：

- ↑↓ 导航选择连接，Enter 连接
- 编辑已有连接（当前值作为默认值）
- 删除连接（需确认）
- 新建连接并自动连接
- Esc 退出 / 返回上一级
- 显示设置：窗口大小、全屏、动态调整、缩放

### 命令行模式

```bash
# 直接连接
rrdp connect <服务器地址> [选项]

# 选项示例
rrdp connect 192.168.1.100 -u administrator --clipboard --fullscreen

# 保存连接配置
rrdp save <名称> -s <服务器> -u <用户名> -d <域名> -c <描述>

# 加载并连接已保存的配置
rrdp load <名称> [-p <密码>]

# 列出已保存的连接
rrdp list

# 删除已保存的连接
rrdp delete <名称>
```

### connect 选项

| 选项 | 说明 |
|---|---|
| `-u`, `--username` | 登录用户名 |
| `-p`, `--password` | 登录密码 |
| `-d`, `--domain` | 域 |
| `--width` | 远程桌面宽度（默认 1920） |
| `--height` | 远程桌面高度（默认 1080） |
| `-f`, `--fullscreen` | 全屏模式 |
| `--clipboard` | 启用剪贴板共享 |
| `--drive <路径>` | 启用驱动器重定向 |
| `--audio` | 启用音频输出 |
| `--nla` | 网络级身份验证 (NLA) |
| `--tls` | TLS 安全连接 |
| `--dynamic-resolution` | 允许动态调整窗口大小 |
| `--scale-desktop <百分比>` | 桌面缩放 (100-500) |
| `--smart-sizing` | 智能缩放以适应窗口 |
| `--` | 附加的 xfreerdp3 原生参数 |

### 显示设置说明

- **动态调整** (`--dynamic-resolution`)：连接后可以自由调整窗口大小，远程桌面会自动适应
- **桌面缩放** (`--scale-desktop`)：设置远程桌面的 DPI 缩放百分比，适合高分屏
- **智能缩放** (`--smart-sizing`)：自动缩放远程桌面以适应窗口大小

### 示例

```bash
# 交互模式 —— 选择/管理连接
rrdp

# 快速连接
rrdp connect 192.168.1.100 -u administrator

# 全屏 + 剪贴板
rrdp connect 10.0.0.5 -u admin --fullscreen --clipboard

# 动态调整窗口 + 缩放
rrdp connect 192.168.1.100 -u admin --dynamic-resolution --scale-desktop 150

# 保存常用连接
rrdp save win10 -s 192.168.1.100 -u administrator -d corp

# 加载连接
rrdp load win10

# 传递原生 xfreerdp3 参数
rrdp connect 192.168.1.100 -- -gfx:avc444 /scale-desktop:150
```

## 数据存储

连接配置保存在：

```
~/.config/rrdp/connections.json
```

格式为 JSON，可直接编辑。密码**不会**被保存到文件中。

## 错误码参考

| 退出码 | 含义 |
|---|---|
| 0 | 成功 |
| 3 | 协议错误 |
| 5 | 访问被拒绝 |
| 6 | 连接被拒绝 |
| 8 | 权限不足 |
| 12 | 连接超时 |
| 18 | TLS 错误 |
| 24 | 身份验证失败（用户名或密码错误） |
| ... | 完整列表见 `connection.rs` 中的 `freerdp_exit_code_hint` |

## 开发

```bash
# 编译（调试模式）
cargo build

# 编译（发布模式）
cargo build --release

# 运行
cargo run -- <子命令>
```

### 项目结构

```
src/
├── main.rs          # 入口、CLI 参数定义、子命令分发
├── config.rs        # 连接配置的序列化/反序列化（JSON）
├── connection.rs    # ConnectionBuilder —— 构建并执行 xfreerdp3 命令
└── interactive.rs   # 交互模式 TUI（dialoguer）
```

## 许可证

MIT