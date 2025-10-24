use std::sync::Mutex;
use crate::Task;



pub(crate) enum TaskSystemModification { Add(Task), Remove(String) }



pub struct TaskScheduler(Mutex<Vec<TaskSystemModification>>);
impl TaskScheduler {

	/// Create a new modifications scheduler.
	pub const fn new() -> TaskScheduler {
		TaskScheduler(Mutex::new(Vec::new()))
	}

	/// Add a task to the system. Does not immediately add it, but puts a request in the queue that adds it on the first run.
	pub fn add_task(&self, task:Task) {
		self.0.lock().unwrap().push(TaskSystemModification::Add(task));
	}

	/// Remove a task by name from the system. Does not immediately remove it, but puts a request in the queue that adds it on the first run.
	pub fn remove_task(&self, task_name:&str) {
		self.0.lock().unwrap().push(TaskSystemModification::Remove(task_name.to_string()));
	}

	/// Extract all requested modifications.
	pub(crate) fn drain(&self) -> Vec<TaskSystemModification> {
		self.0.lock().unwrap().drain(..).collect()
	}
}