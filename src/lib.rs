mod task;
mod task_event;
mod task_system;
mod task_system_u;
mod task_u;
mod task_scheduler;
mod task_scheduler_u;

pub use task::*;
pub use task_event::*;
pub use task_system::*;
pub use task_scheduler::*;



use std::sync::Mutex;
pub static STATIC_TASK_SYSTEM:Mutex<TaskSystem> = Mutex::new(TaskSystem::new());