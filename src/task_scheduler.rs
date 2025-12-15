use std::sync::{ Mutex, MutexGuard };
use crate::{ TaskLike, TaskType };



type TaskRetainFilter = Box<dyn Fn(&dyn TaskLike) -> bool + Send + Sync + 'static>;



pub(crate) enum TaskSystemModification { Add(Box<dyn TaskLike + Send + Sync + 'static>), RetainTasks(TaskRetainFilter), TriggerEvent(String) }



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

	/// Retain tasks by the given filter. Does not immediately remove them, but puts a request in the queue that adds it on the next update of the system.
	pub fn retain_tasks<T:Fn(&dyn TaskLike) -> bool + Send + Sync + 'static>(&self, filter:T) {
		self.add_modification(TaskSystemModification::RetainTasks(Box::new(filter)));
	}

	/// Remove a task by name from the system. Does not immediately remove it, but puts a request in the queue that adds it on the next update of the system.
	pub fn remove_task(&self, task_name:&str) {
		let task_name:String = task_name.to_string();
		self.retain_tasks(move |task| task.name() != task_name);
	}

	/// Remove all scheduled tasks. Keeps subscription tasks. Does not immediately trigger it, but puts a request in the queue that triggers it on the next update of the system.
	pub fn remove_scheduled_tasks(&self) {
		self.retain_tasks(move |task| task.task_type() != TaskType::Task);
	}

	/// Trigger an event, activating all its subscriptions. Does not immediately trigger it, but puts a request in the queue that triggers it on the next update of the system.
	pub fn trigger_event(&self, event_name:&str) {
		self.add_modification(TaskSystemModification::TriggerEvent(event_name.to_string()));
	}

	/// Extract all requested modifications.
	pub(crate) fn drain(&self) -> Vec<TaskSystemModification> {
		self.0.lock().unwrap().drain(..).collect()
	}

	/// Drain specifically the named events.
	pub fn drain_named_events(&self) -> Vec<String> {
		let mut events_handle:MutexGuard<'_, Vec<TaskSystemModification>> = self.0.lock().unwrap();
		let mut named_events:Vec<String> = Vec::new();
		let mut index:usize = 0;
		while index < events_handle.len() {
			match &events_handle[index] {
				TaskSystemModification::TriggerEvent(event_name) => {
					named_events.push(event_name.to_string());
					events_handle.remove(index);
				},
				_ => index += 1
			}
		}
		named_events
	}
}
impl Default for TaskScheduler {
	fn default() -> Self {
		TaskScheduler::new()
	}
}