use std::{ error::Error, sync::{ Arc, Mutex, MutexGuard }, thread::{ self, JoinHandle, sleep }, time::{ Duration, Instant } };
use crate::{ Task, TaskModificationRequest, TaskScheduler };



#[derive(PartialEq)]
enum TaskSystemStatus { Running, StopRequested, Idle }
const MAX_INTERVAL:Duration = Duration::from_millis(10);


pub struct TaskSystem {
	tasks:Vec<Task>, // Tasks are passed to another thread in the running method, so this does not need any synchronization technique.
	scheduler:Arc<TaskScheduler>,
	error_handler:Arc<dyn Fn(&str, Box<dyn Error>) + Send + Sync + 'static>,
	status:Arc<Mutex<TaskSystemStatus>>,
	thread_handle:Option<JoinHandle<Vec<Task>>>
}
impl TaskSystem {

	/* CONSTRUCTOR METHODS */

	/// Create a new system.
	pub fn new() -> TaskSystem {
		TaskSystem::default()
	}



	/* RUNNING METHODS */

	/// Start the system, runs it in another thread.
	/// If the system is already running, this will do nothing.
	pub fn start(&mut self) {
		if self.thread_handle.is_none() {
			self.thread_handle = Some(self.spawn_thread(|| true));
		}
	}

	/// Stop the system if it already running.
	/// If the system is not running, will do nothing.
	/// Can fail if the thread has panicked.
	pub fn stop(&mut self) {
		let mut status_handle:MutexGuard<'_, TaskSystemStatus> = self.status.lock().unwrap();
		if *status_handle == TaskSystemStatus::Running && self.thread_handle.is_some() {
			*status_handle = TaskSystemStatus::StopRequested;
			drop(status_handle);
			self.join_thread();
		}
	}

	/// Keep running the system indefinitely.
	/// Will not do anything if the system is already running async.
	pub fn run(&mut self) {
		self.run_while(|| true);
	}

	/// Run the system once.
	pub fn run_once(&mut self) {
		let mut index:usize = 0;
		self.run_while(move || {
			index += 1;
			index == 1
		});
	}

	/// Run while the given condition is true.
	pub fn run_while<RunWhileCondition:FnMut() -> bool + Send + Sync + 'static>(&mut self, run_while_condition:RunWhileCondition) {
		if self.thread_handle.is_none() {
			self.thread_handle = Some(self.spawn_thread(run_while_condition));
		}
		self.join_thread();
	}

	/// Spawn the thread that handles the tasks.
	fn spawn_thread<RunWhileCondition:FnMut() -> bool + Send + Sync + 'static>(&mut self, mut run_while_condition:RunWhileCondition) -> JoinHandle<Vec<Task>> {
		*self.status.lock().unwrap() = TaskSystemStatus::Running;
		let mut thread_tasks:Vec<Task> = self.tasks.drain(..).collect();
		let thread_scheduler:Arc<TaskScheduler> = Arc::clone(&self.scheduler);
		let thread_status_handle:Arc<Mutex<TaskSystemStatus>> = Arc::clone(&self.status);
		let thread_error_handler:Arc<dyn Fn(&str, Box<dyn Error + 'static>) + Send + Sync> = Arc::clone(&self.error_handler);
		thread::spawn(move || {

			// Keep updating and running tasks until a stop has been requested.
			while run_while_condition() && *thread_status_handle.lock().unwrap() == TaskSystemStatus::Running {

				// Apply modifications in the scheduler queue.
				for modification in thread_scheduler.drain() {
					match modification {
						TaskModificationRequest::AddTask(task) => thread_tasks.push(task),
						TaskModificationRequest::RetainTasks(filter) => thread_tasks.retain(filter)
					}
				}

				// Run all tasks and store errors.
				let now:Instant =  Instant::now();
				for task in &mut thread_tasks {
					if let Err(error) = task.run(&now) { // The run method skips any tasks that are not due to run.
						thread_error_handler(&task.name, error);
					}
				}

				// Remove expired tasks.
				thread_tasks.retain(|task| !task.event.expired);

				// Sleep until next interval.
				let now:Instant = Instant::now();
				let next_target:Instant = thread_tasks.iter().map(|task| task.event.trigger_target).min().unwrap_or(now + MAX_INTERVAL);
				if next_target > now {
					sleep(next_target.duration_since(now));
				}
			}

			// Update the status and return the original tasks, allowing the idle system to have them again.
			*thread_status_handle.lock().unwrap() = TaskSystemStatus::Idle;
			thread_tasks
		})
	}

	/// Join the thread that handles the tasks and restore the task back to this struct.
	/// Will print an error and drop all stored tasks if something went wrong.
	fn join_thread(&mut self) {
		if let Some(handle) = self.thread_handle.take() {
			match handle.join() {
				Ok(tasks) => self.tasks = tasks,
				Err(error) => eprintln!("TaskSystem runner thread exit failed: {:?}", error)
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
			error_handler: Arc::new(|task_name, error| eprintln!("TaskSyncer task '{task_name}' panicked: {error}")),
			status: Arc::new(Mutex::new(TaskSystemStatus::Idle)),
			thread_handle: None
		}
	}
}