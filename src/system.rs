use std::{ error::Error, time::Instant, sync::{ Arc, Mutex }, thread::{ self, JoinHandle } };
use modifications_queue::{ ModificationsQueue, ModificationsQueueRemote };
use crate::Task;



pub struct TaskSystem {
	built_in_scheduler:TaskScheduler,
	error_handler:Arc<dyn Fn(&str, Box<dyn Error>) + Send + Sync + 'static>,
	is_running:Arc<Mutex<bool>>,
	_thread_handle:JoinHandle<()>
}
impl TaskSystem {

	/* CONSTRUCTOR METHODS */

	/// Create a new system.
	pub fn new() -> TaskSystem {
		TaskSystem::default()
	}

	/// Return the system with an overwritten error handler.
	/// The handler gets the name of the profile as the first argument and the generated error as the second.
	pub fn with_error_handler<ErrorHandler:Fn(&str, Box<dyn Error>) + Send + Sync + 'static>(mut self, error_handler:ErrorHandler) -> Self {
		self.set_error_handler(error_handler);
		self
	}

	/// Overwrite the error handler.
	/// The handler gets the name of the profile as the first argument and the generated error as the second.
	pub fn set_error_handler<ErrorHandler:Fn(&str, Box<dyn Error>) + Send + Sync + 'static>(&mut self, error_handler:ErrorHandler) {
		self.error_handler = Arc::new(error_handler);
	}



	/* RUNNING METHODS */

	/// Start the system.
	/// Signals the linked thread to start running the tasks.
	/// If the system is already running, this will do nothing.
	pub fn start(&mut self) {
		*self.is_running.lock().unwrap() = true;
	}

	/// Stop the system if it already running.
	/// Signals the linked thread to stop running tasks.
	/// If the system is not running, will do nothing.
	pub fn stop(&mut self) {
		*self.is_running.lock().unwrap() = false;
	}

	/// Spawn the thread that handles all modifications and tasks.
	fn spawn_thread(modifications_queue:ModificationsQueue<(Vec<Task>, Arc<Mutex<bool>>)>, status:Arc<Mutex<bool>>, error_handler:Arc<dyn Fn(&str, Box<dyn Error>) + Send + Sync + 'static>) -> JoinHandle<()> {
		thread::spawn(move || {
			let mut tasks_and_status:(Vec<Task>, Arc<Mutex<bool>>) = (Vec::new(), status);

			// Handle modifications until the system should run.
			while !*tasks_and_status.1.lock().unwrap() {
				for modification in modifications_queue.await_change() {
					modification(&mut tasks_and_status);
				}
			}

			// Handle modifications and tasks while the system should run.
			while *tasks_and_status.1.lock().unwrap() {

				// Run all tasks.
				let now:Instant =  Instant::now();
				for task in &mut tasks_and_status.0 {
					if let Err(error) = task.run(&now) { // The run method skips any tasks that are not due to run.
						error_handler(&task.name, error);
					}
				}

				// Wait until the next task is scheduled or the next modification.
				let next_task_target:Option<Instant> = tasks_and_status.0.iter().map(|task| task.event.trigger_target).min();
				let modifications:Vec<Box<dyn FnOnce(&mut (Vec<Task>, Arc<Mutex<bool>>)) + Send + Sync>> = {
					match next_task_target {
						Some(next_trigger_target) => {
							let now:Instant = Instant::now();
							if next_trigger_target <= now {
								modifications_queue.drain()
							} else {
								modifications_queue.await_change_timeout(next_trigger_target - now)
							}
						},
						None => modifications_queue.await_change()
					}
				};
				for modification in modifications {
					modification(&mut tasks_and_status);
				}
			}

		})
	}



	/* SCHEDULER METHODS */

	/// Get the scheduler of the system.
	pub fn scheduler(&self) -> TaskScheduler {
		self.built_in_scheduler.clone()
	}

	/// Add a task to the system.
	pub fn add_task(&self, task:Task) {
		self.built_in_scheduler.add_task(task);
	}

	/// Request to add a new task to the system, overwriting any existing ones with the same name. Will be applied on the next run of the system.
	pub fn update_task(&self, task:Task) {
		self.built_in_scheduler.update_task(task);
	}

	/// Request to remove a task from the system. Will be applied on the next run of the system.
	pub fn remove_task(&self, task_name:&str) {
		self.built_in_scheduler.remove_task(task_name);
	}

	/// Retain tasks by a specific filter.
	pub fn retain_tasks<Filter:FnMut(&Task) -> bool + Send + Sync + 'static>(&self, filter:Filter) {
		self.built_in_scheduler.retain_tasks(filter);
	}
}
impl Default for TaskSystem {
	fn default() -> Self {
		let modifications_queue:ModificationsQueue<(Vec<Task>, Arc<Mutex<bool>>)> = ModificationsQueue::new();
		let status:Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
		let error_handler:Arc<dyn Fn(&str, Box<dyn Error>) + Send + Sync + 'static> = Arc::new(|task_name, error| eprintln!("TaskSyncer task '{task_name}' panicked: {error}"));
		TaskSystem {
			built_in_scheduler: TaskScheduler(modifications_queue.create_remote()),
			is_running: Arc::clone(&status),
			error_handler: Arc::clone(&error_handler),
			_thread_handle: TaskSystem::spawn_thread(modifications_queue, status, error_handler)
		}
	}
}



#[derive(Clone)]
pub struct TaskScheduler(ModificationsQueueRemote<(Vec<Task>, Arc<Mutex<bool>>)>);
impl TaskScheduler {
	
	/// Request to add a new task to the system. Will be applied on the next run of the system.
	pub fn add_task(&self, task:Task) {
		self.0.add(move |(tasks, _)| tasks.push(task));
	}

	/// Request to add a new task to the system, overwriting any existing ones with the same name. Will be applied on the next run of the system.
	pub fn update_task(&self, task:Task) {
		self.0.add(move |(tasks, _)| {
			for existing_task in &mut **tasks {
				if existing_task.name == task.name {
					*existing_task = task;
					break;
				}
			}
		});
	}

	/// Request to remove a task from the system. Will be applied on the next run of the system.
	pub fn remove_task(&self, task_name:&str) {
		let task_name:String = task_name.to_string();
		self.retain_tasks(move |task| task.name != task_name);
	}

	/// Retain tasks by a specific filter.
	pub fn retain_tasks<Filter:FnMut(&Task) -> bool + Send + Sync + 'static>(&self, filter:Filter) {
		self.0.add(move |(tasks, _)| tasks.retain(filter));
	}
}