# ib-rust

Interactive Brokers Gateway + Rust 交易引擎。首次使用：`cp .env.example .env` 并填入 IB 账号。

Rust 引擎架构与模块说明见 [rust-engine/README.md](rust-engine/README.md)。

```bash
# 1. Gateway
docker compose up ib-gateway

# 2. 生产
docker compose --profile prod up -d --build

# 3. 开发
docker compose --profile dev up -d --build rust-dev
```

### IB Gateway VNC（2FA / 登录界面）

在 `.env` 里设置 `VNC_SERVER_PASSWORD` 后，Gateway 会把 VNC 映射到本机 `127.0.0.1:5900`。

**Mac（命令行打开系统「屏幕共享」）：**

```bash
open vnc://127.0.0.1:5900
```

提示输入密码时，填 `.env` 里的 `VNC_SERVER_PASSWORD`（不是 IB 登录密码）。

**可选 — TigerVNC 纯命令行客户端：**

```bash
brew install tiger-vnc-viewer
vncviewer 127.0.0.1:5900
```

