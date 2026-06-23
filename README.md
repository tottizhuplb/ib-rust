# ib-rust

基于 Docker 的 Interactive Brokers Gateway + Rust 数据采集服务。

Gateway 镜像：[gnzsnz/ib-gateway-docker](https://github.com/gnzsnz/ib-gateway-docker)（`ghcr.io/gnzsnz/ib-gateway:stable`）

## 项目结构

```
.
├── docker-compose.yml      # 服务编排（ib-gateway / rust-collector）
├── .env.example            # 环境变量模板
├── ib-gateway/             # Gateway 本地数据（与 compose 服务名一致，可扩展更多 mount）
│   └── tws_settings/       # 设置持久化 → 容器 ${TWS_SETTINGS_PATH}
└── rust-collector/         # Rust 采集器源码
    ├── Dockerfile          # 生产镜像（release 二进制）
    ├── Dockerfile.dev      # 开发镜像（Rust 工具链 + cargo-watch）
    └── src/
```

## 前置条件

- [Docker](https://docs.docker.com/get-docker/) 与 Docker Compose
- Interactive Brokers 账号

## 首次配置

```bash
cp .env.example .env
```

编辑 `.env`，至少填入 IB 账号（其余默认值见 `.env.example`）：

```bash
TWS_USERID=你的IB用户名
TWS_PASSWORD=你的IB密码
```

启用 VNC 时在 `.env` 中设置：

```bash
VNC_SERVER_PASSWORD=你的VNC密码
```

VNC 地址：`127.0.0.1:5900`

## 启动方式

### 开发模式（推荐）

挂载本地源码，在容器内开发。本机无需安装 Rust。

```bash
# 启动开发容器（需 ib-gateway 已 healthy）
docker compose --profile dev up -d --build rust-dev

# 进入容器（推荐）
docker exec -it rust-dev bash
```

容器内常用命令：

```bash
cargo run          # 运行 collector
cargo build        # 编译
cargo test         # 测试
cargo watch -x run # 改代码自动重跑（可选）
```

- 源码 `./rust-collector` 挂载到容器 `/app`，宿主机改代码容器内即时可见
- 依赖与编译缓存保存在 Docker volume，避免 macOS bind mount 编译过慢

停止：

```bash
docker compose --profile dev down
```

### 生产模式

将 release 二进制打包进镜像，适合部署验证。

```bash
docker compose --profile prod up -d --build
```

查看日志：

```bash
docker compose logs -f rust-collector
```

停止：

```bash
docker compose --profile prod down
```

### 仅启动 IB Gateway

不启动 collector，只跑 Gateway：

```bash
docker compose up ib-gateway
```

## 端口说明

gnzsnz 镜像通过 socat 转发 API 端口，宿主机仍使用标准 IB 端口：

| 宿主机端口 | 容器 socat 端口 | 用途 |
|-----------|----------------|------|
| 4001 | 4003 | IB Gateway 实盘 API |
| 4002 | 4004 | IB Gateway 模拟盘 API |
| 5900 | 5900 | VNC（需配置 `VNC_SERVER_PASSWORD`） |

容器网络内（rust-collector → ib-gateway）使用 socat 端口：paper **4004**，live **4003**（collector 已自动处理）。

## 环境变量

所有配置集中在 `.env`（从 `.env.example` 复制），Gateway 与 collector **共用同一文件**（`env_file: .env`）。

完整变量列表与默认值见 [`.env.example`](.env.example)。

## 常见问题

**从旧版 datawookie 镜像切换**

需重新拉取镜像并重建容器：

```bash
docker compose down
docker compose pull ib-gateway
docker compose up ib-gateway
```

**Gateway 启动后 collector 连接失败**

Gateway 首次登录可能需要 2FA。配置 VNC 密码后，通过 VNC 客户端连接 `127.0.0.1:5900` 完成验证，等待 healthcheck 通过后 collector 会自动连接。

**`docker compose up` 只启动了 ib-gateway**

这是预期行为。collector 需要通过 profile 显式启动：

- 开发：`docker compose --profile dev up`
- 生产：`docker compose --profile prod up -d`
