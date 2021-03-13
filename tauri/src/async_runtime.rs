use once_cell::sync::OnceCell;
use tokio::runtime::Runtime;
pub use tokio::sync::Mutex;

use std::{future::Future, sync::Mutex as StdMutex};

static RUNTIME: OnceCell<StdMutex<Runtime>> = OnceCell::new();

pub fn block_on<F: Future>(task: F) -> F::Output {
  let runtime = RUNTIME.get_or_init(|| StdMutex::new(Runtime::new().unwrap()));
  runtime.lock().unwrap().block_on(task)
}

pub fn spawn<F>(task: F)
where
  F: Future + Send + 'static,
  F::Output: Send + 'static,
{
  let runtime = RUNTIME.get_or_init(|| StdMutex::new(Runtime::new().unwrap()));
  runtime.lock().unwrap().spawn(task);
}
