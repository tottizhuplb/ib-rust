# rust-engine

事件驱动的模块化单体：连接 IB Gateway，采集行情并本地落盘。当前只实现 **market** 域；**strategy / risk / order** 预留占位。

主链（目标）：`market → strategy → risk → order`

## 域划分

| 域 | 状态 | 一句话 |
|----|------|--------|
| **market** | 已实现 | IB 连接、订阅、事件标准化、内存盘口、分段写盘、快照、健康检查 |
| **strategy** | 预留 | 消费 market 输出，产生交易信号或下单意图 |
| **risk** | 预留 | 对 strategy 交易意图做预交易审查与约束（限额、kill switch） |
| **order** | 预留 | 订单生命周期与 broker 执行 |

## 目录

```
src/
├── main.rs / app.rs     # 装配与 run_forever()
├── config/              # config.yaml + 环境变量覆盖
├── core/                # 共享事件契约（domain / pipeline / RunState）
├── market/              # 当前唯一实现域
├── strategy/            # 预留
├── risk/                # 预留
└── order/               # 预留
```

## 启动

```bash
# 编辑 config.yaml 后
cargo run

# 或开发热重载
cargo watch -x run
```

配置：`config.yaml`（主配置）+ 环境变量覆盖（如 `TRADING_MODE`、`IB_HOST=ib-gateway`）。

## 依赖方向

```
app → market / config
market/* → core
config → core::domain
```
