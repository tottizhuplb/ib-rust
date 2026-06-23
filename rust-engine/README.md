# rust-engine

事件驱动的模块化单体：连接 IB Gateway，采集行情并本地落盘。当前只实现 **market** 域；**strategy / risk / order** 预留占位。

开发与生产均在 **Docker（Linux）** 内运行；单进程、单 Tokio runtime，各域通过进程内 channel 通信。

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
conf/
├── config.yaml                 # 入口配置（基础设施 + 外部文件路径）
└── market/
    └── subscriptions.yaml      # 订阅标的

src/
├── main.rs                     # 入口、信号、graceful shutdown
├── logging.rs                  # 文件 + stdout 日志（含线程名）
├── core/
│   ├── config/                 # conf/ yaml 加载
│   ├── model/                  # 跨域共享类型（MarketEvent、Symbol…）
│   ├── pipeline/               # EventPublisher、bounded channel
│   ├── task.rs                 # TaskGroup · wait_for_signal_or_worker · EngineStop
│   └── run_state.rs            # 进程编排状态（非落盘事件）
├── market/
│   ├── runtime.rs              # register() 向顶层 TaskGroup 注册 worker
│   ├── config/                 # IB / 存储 / 管道配置模型
│   ├── connection/             # supervisor、session、IB adapter
│   ├── subscription/           # desired → active reconcile
│   ├── recorder/               # 原始事件 jsonl.zst 落盘
│   ├── state/                  # 内存盘口
│   ├── snapshot/               # 周期导出
│   └── health/
├── strategy/                   # 预留
├── risk/                       # 预留
└── order/                      # 预留
```

## 配置

看 `conf/config.yaml` 和 `conf/market/subscriptions.yaml` 即可；yaml 里已有注释。运行时还需环境变量 `TRADING_MODE`（`paper` / `live`）。

## 启动

```bash
# 编辑 conf/config.yaml 与 conf/market/subscriptions.yaml 后
cargo run

# 开发热重载
cargo watch -x run

# 格式化（编辑器保存时会自动 fmt；提交前或批量改代码后可手动跑）
cargo fmt --all
```

二进制名：`engine`（`Cargo.toml` 中 `[[bin]] name = "engine"`）。

生产镜像入口为 `CMD ["./engine"]`；停服时 Docker 向 PID 1 发 **SIGTERM**。

## 运行时结构

`main.rs` 持有一个顶层 [`TaskGroup`](src/core/task.rs)，各域 `register` 注册 worker；`wait_for_signal_or_worker` 统一等信号或 worker 退出。

| Worker（async task） | 职责 |
|--------------------|------|
| `market-connection` | IB 连接 supervisor，写 `RunState`，跑 session reader |
| `market-subscription` | 读 `RunState`，`Connected` 时 reconcile 订阅 |
| `market-recorder` | 消费 `MarketEvent` mpsc，单写盘 jsonl.zst |
| `market-snapshot` | 周期读内存盘口导出（当前 stub） |
| `market-health` | 周期 health tick |

`RunState`（`watch`）是进程内编排状态；`MarketEvent::Connection` 是落盘/下游用的领域事件，两者职责不同。

## 并发模型

不是「一个 worker 一个 OS 线程」：**只有 main task 监听 OS 信号**；各域 worker 注册进同一个顶层 `TaskGroup`，由 `wait_for_signal_or_worker` 统一 `join`。

```mermaid
flowchart TB
  subgraph proc["1 进程 · 1 Tokio Runtime · tokio-runtime-worker × N"]
    main["main task ×1<br/>SIGTERM / SIGINT<br/>wait_for_signal_or_worker"]
    tg["TaskGroup ×1"]
    main <-->|"join_next"| tg

    subgraph reg["各域 register → spawn_named"]
      mk["market ×5"]
      fu["strategy / risk / order …"]
    end

    tg --> reg
  end

  sig["docker stop / Ctrl+C"] --> main
  fail["任一 worker Err / panic"] --> main
  main -->|"begin_shutdown · drain 全家"| tg
```

触发 shutdown 后：`broadcast` 通知 worker → 各域 `begin_shutdown` → `TaskGroup::drain`。worker 失败也会走完整 drain，进程最后以非 0 退出（`EngineStop::worker_error`）。
