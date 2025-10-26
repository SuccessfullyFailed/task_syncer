use std::sync::Mutex;
use crate::TaskLike;



pub(crate) enum TaskSystemModification { Add(Box<dyn TaskLike + Send + Sync>), Remove(String), TriggerEvent(String) }



pub struct TaskScheduler(Mutex<Vec<TaskSystemModification>>);
impl TaskScheduler {

	/// Create a new modifications scheduler.
	pub const fn new() -> TaskScheduler {
		TaskScheduler(Mutex::new(Vec::new()))
	}

	/// Add a modification to the scheduler list.
	fn add_modification(&self, modification:TaskSystemModification) {
		self.0.lock().unwrap().push(modification);
	}

	/// Add a task to the system. Does not immediately add it, but puts a request in the queue that adds it on the next update of the system.
	pub fn add_task<T:TaskLike + Send + Sync + 'static>(&self, task:T) {
		self.add_modification(TaskSystemModification::Add(Box::new(task)));
	}

	/// Remove a task by name from the system. Does not immediately remove it, but puts a request in the queue that adds it on the next update of the system.
	pub fn remove_task(&self, task_name:&str) {
		self.add_modification(TaskSystemModification::Remove(task_name.to_string()));
	}

	/// Trigger an event, activating all its subscriptions. Does not immediately trigger it, but puts a request in the queue that triggers it on the next update of the system.
	pub fn trigger_event(&self, event_name:&str) {
		self.add_modification(TaskSystemModification::TriggerEvent(event_name.to_string()));
	}

	/// Extract all requested modifications.
	pub(crate) fn drain(&self) -> Vec<TaskSystemModification> {
		self.0.lock().unwrap().drain(..).collect()
	}
}