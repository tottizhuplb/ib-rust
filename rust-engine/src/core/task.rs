use std::future::Future;

use tokio::task::{JoinError, JoinSet};
use tracing::Instrument;

pub type TaskResult = anyhow::Result<()>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    Signal,
    TaskExited,
    WorkerFailed,
}

/// 触发 shutdown 的原因；worker 失败时仍走 graceful shutdown，错误留到 drain 之后返回。
#[derive(Debug)]
pub struct EngineStop {
    pub reason: StopReason,
    pub worker_error: Option<anyhow::Error>,
}

impl EngineStop {
    pub fn into_result(self) -> anyhow::Result<()> {
        match self.worker_error {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }
}

/// 顶层 worker 集合；各域通过 [`TaskGroup::spawn_named`] 注册 task。
pub struct TaskGroup {
    set: JoinSet<TaskResult>,
}

impl TaskGroup {
    pub fn new() -> Self {
        Self {
            set: JoinSet::new(),
        }
    }

    pub fn spawn_named(
        &mut self,
        name: &'static str,
        future: impl Future<Output = TaskResult> + Send + 'static,
    ) {
        self.set.spawn(
            async move { future.await }.instrument(tracing::info_span!("worker", worker = name)),
        );
    }

    pub async fn join_next(&mut self) -> Option<Result<TaskResult, JoinError>> {
        self.set.join_next().await
    }

    pub async fn drain(&mut self) {
        while let Some(res) = self.join_next().await {
            match res {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    tracing::warn!(error = %e, "worker stopped with error during shutdown")
                }
                Err(e) => tracing::warn!(error = %e, "task join error during shutdown"),
            }
        }
    }
}

impl Default for TaskGroup {
    fn default() -> Self {
        Self::new()
    }
}

/// 等待 OS 信号或任一 worker 退出；全进程共用此循环。
/// worker `Err` / panic 不会跳过 shutdown，错误保存在 [`EngineStop::worker_error`]。
pub async fn wait_for_signal_or_worker<F>(
    signal: F,
    tasks: &mut TaskGroup,
) -> anyhow::Result<EngineStop>
where
    F: Future<Output = anyhow::Result<()>>,
{
    tokio::select! {
        res = signal => {
            res?;
            Ok(EngineStop {
                reason: StopReason::Signal,
                worker_error: None,
            })
        }
        worker_join = tasks.join_next() => match worker_join {
            Some(Ok(Ok(()))) => Ok(EngineStop {
                reason: StopReason::TaskExited,
                worker_error: None,
            }),
            Some(Ok(Err(e))) => {
                tracing::error!(error = %e, "worker failed, shutting down");
                Ok(EngineStop {
                    reason: StopReason::WorkerFailed,
                    worker_error: Some(e),
                })
            }
            Some(Err(e)) => {
                tracing::error!(error = %e, "worker panicked, shutting down");
                Ok(EngineStop {
                    reason: StopReason::WorkerFailed,
                    worker_error: Some(anyhow::anyhow!("task join error: {e}")),
                })
            }
            None => anyhow::bail!("TaskGroup has no workers; expected long-running tasks until signal or failure"),
        },
    }
}
