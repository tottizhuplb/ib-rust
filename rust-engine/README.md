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
├── core/
│   ├── config/          # conf/ yaml 加载
│   ├── model/           # 跨域共享类型（MarketEvent、Symbol…）
│   └── pipeline/
├── market/
│   ├── config/          # IB / 存储 / 管道配置模型
│   └── subscription/    # 订阅业务模型
├── strategy/            # 预留
├── risk/                # 预留
└── order/               # 预留
```

## 启动

```bash
# 编辑 conf/config.yaml 后
cargo run

# 或开发热重载
cargo watch -x run

# 格式化（编辑器保存时会自动 fmt；提交前或批量改代码后可手动跑）
cargo fmt --all
```

配置：`conf/config.yaml`（主配置）+ `conf/<domain>/` 业务配置 + 环境变量覆盖（如 `TRADING_MODE`、`IB_HOST=ib-gateway`）。

## 依赖方向

```
app → core / market
market/* → core
core/config → market/config（组装 MarketConfig）
```
