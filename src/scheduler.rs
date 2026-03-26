use std::sync::Mutex;
use crate::Task;



pub enum TaskModificationRequest { AddTask(Task), RetainTasks(Box<dyn FnMut(&Task) -> bool + Send + Sync + 'static>) }
pub struct TaskScheduler(Mutex<Vec<TaskModificationRequest>>);
impl TaskScheduler {

	/// Request to add a new task to the system. Will be applied on the next run of the system.
	pub fn add_task(&self, task:Task) {
		self.0.lock().unwrap().push(TaskModificationRequest::AddTask(task));
	}

	/// Request to add a new task to the system, overwriting any existing ones with the same name. Will be applied on the next run of the system.
	pub fn update_task(&self, task:Task) {
		self.remove_task(&task.name);
		self.add_task(task);
	}

	/// Request to remove a task from the system. Will be applied on the next run of the system.
	pub fn remove_task(&self, task_name:&str) {
		let task_name:String = task_name.to_string();
		self.retain_tasks(move |task| task.name != task_name);
	}

	/// Retain tasks by a specific filter.
	pub fn retain_tasks<Filter:FnMut(&Task) -> bool + Send + Sync + 'static>(&self, filter:Filter) {
		self.0.lock().unwrap().push(TaskModificationRequest::RetainTasks(Box::new(filter)));
	}

	/// Get all the tasks, empties the list.
	pub(crate) fn drain(&self) -> Vec<TaskModificationRequest> {
		self.0.lock().unwrap().drain(..).collect()
	}
}
impl Default for TaskScheduler {
	fn default() -> Self {
		TaskScheduler(Mutex::new(Vec::new()))
	}
}