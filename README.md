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
