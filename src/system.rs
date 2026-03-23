use std::{ error::Error, sync::{ Arc, Mutex }, thread::{ self, JoinHandle, sleep }, time::{ Duration, Instant } };
use crate::{ Task, TaskModificationRequest, TaskScheduler };



#[derive(PartialEq)]
enum TaskSystemStatus { Running, StopRequested, Stopped }
const MAX_INTERVAL:Duration = Duration::from_millis(250);



pub struct TaskSystem {
	tasks:Vec<Task>, // Tasks are only modified in the running method, so this does not need any synchronization technique.
	scheduler:Arc<TaskScheduler>,
	error_handler:Box<dyn Fn(&str, Box<dyn Error>) + Send + Sync + 'static>
}
impl TaskSystem {

	/* CONSTRUCTOR METHODS */

	/// Create a new system.
	pub fn new() -> TaskSystem {
		TaskSystem::default()
	}



	/* RUNNING METHODS */

	/// Start the system, runs it in another thread.
	pub fn start(self) -> TaskSystemRunHandle {
		TaskSystemRunHandle::new(self)
	}

	/// Keep running the system indefinitely.
	pub fn run(&mut self) {
		self.run_while(|_| true);
	}

	/// Keep running the system while the given condition is true.
	pub fn run_while<Condition:Fn(&TaskSystem) -> bool>(&mut self, condition:Condition) {
		while condition(self) {

			// Run system once.
			self.run_once();

			// Sleep until next interval.
			let now:Instant =  Instant::now();
			let next_target:Instant = self.tasks.iter().map(|task| task.event.trigger_target).min().unwrap_or(now + MAX_INTERVAL);
			if next_target > now {
				sleep(next_target.duration_since(now));
			}
		}
	}

	/// Run the system once, updating all tasks once.
	pub fn run_once(&mut self) {
		let now:Instant = Instant::now();

		// Run all tasks and store errors.
		for task in &mut self.tasks {
			if let Err(error) = task.run(&now) {
				(self.error_handler)(&task.name, error);
			}
		}

		// Remove invalid tasks.
		self.tasks.retain(|task| !task.event.expired);

		// Apply modifications in the scheduler queue.
		for modification in self.scheduler.drain() {
			match modification {
				TaskModificationRequest::AddTask(task) => self.tasks.push(task),
				TaskModificationRequest::RetainTasks(filter) => self.tasks.retain(filter)
			}
		}
	}



	/* SCHEDULER METHODS */

	/// Get the scheduler of the system.
	pub fn scheduler(&self) -> &TaskScheduler {
		&self.scheduler
	}

	/// Add a task to the system.
	pub fn add_task(&self, task:Task) {
		self.scheduler.add_task(task);
	}

	/// Request to add a new task to the system, overwriting any existing ones with the same name. Will be applied on the next run of the system.
	pub fn update_task(&self, task:Task) {
		self.scheduler.update_task(task);
	}

	/// Request to remove a task from the system. Will be applied on the next run of the system.
	pub fn remove_task(&self, task_name:&str) {
		self.scheduler.remove_task(task_name);
	}

	/// Retain tasks by a specific filter.
	pub fn retain_tasks<Filter:FnMut(&Task) -> bool + Send + Sync + 'static>(&self, filter:Filter) {
		self.scheduler.retain_tasks(filter);
	}
}
impl Default for TaskSystem {
	fn default() -> Self {
		TaskSystem {
			tasks: Vec::new(),
			scheduler: Arc::new(TaskScheduler::default()),
			error_handler: Box::new(|task_name, error| eprintln!("task '{task_name}' returned error: '{error}'"))
		}
	}
}



pub struct TaskSystemRunHandle {
	thread:JoinHandle<TaskSystem>,
	scheduler:Arc<TaskScheduler>,
	status_handle:Arc<Mutex<TaskSystemStatus>>
}
impl TaskSystemRunHandle {

	/* CREATE/UPDATE METHODS */

	/// Create a new handle.
	pub fn new(mut system:TaskSystem) -> TaskSystemRunHandle {
		let scheduler_clone:Arc<TaskScheduler> = Arc::clone(&system.scheduler);
		let status_handle:Arc<Mutex<TaskSystemStatus>> = Arc::new(Mutex::new(TaskSystemStatus::Running));
		let status_handle_clone:Arc<Mutex<TaskSystemStatus>> = Arc::clone(&status_handle);
		TaskSystemRunHandle {
			thread: thread::spawn(move || {
				system.run_while(|_| *status_handle.lock().unwrap() == TaskSystemStatus::Running);
				*status_handle.lock().unwrap() = TaskSystemStatus::Stopped;
				system
			}),
			scheduler: scheduler_clone,
			status_handle: status_handle_clone
		}
	}

	/// Check if the system is still running.
	pub fn running(&self) -> bool {
		*self.status_handle.lock().unwrap() != TaskSystemStatus::Stopped
	}

	/// Stop the system.
	/// Waits until the remote thread has confirmed exit.
	pub fn stop(self) -> Result<TaskSystem, Box<dyn Error>> {
		*self.status_handle.lock().unwrap() = TaskSystemStatus::StopRequested;
		match self.thread.join() {
			Ok(system) => Ok(system),
			Err(error) => Err(format!("Could not retrieve TaskSystem from run handle: {:?}", error).into())
		}
	}



	/* SCHEDULER METHODS */

	/// Get the scheduler of the system.
	pub fn scheduler(&self) -> &TaskScheduler {
		&self.scheduler
	}

	/// Add a task to the system.
	pub fn add_task(&self, task:Task) {
		self.scheduler.add_task(task);
	}

	/// Request to add a new task to the system, overwriting any existing ones with the same name. Will be applied on the next run of the system.
	pub fn update_task(&self, task:Task) {
		self.scheduler.update_task(task);
	}

	/// Request to remove a task from the system. Will be applied on the next run of the system.
	pub fn remove_task(&self, task_name:&str) {
		self.scheduler.remove_task(task_name);
	}

	/// Retain tasks by a specific filter.
	pub fn retain_tasks<Filter:FnMut(&Task) -> bool + Send + Sync + 'static>(&self, filter:Filter) {
		self.scheduler.retain_tasks(filter);
	}
}