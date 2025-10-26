use std::{ sync::{ Mutex, MutexGuard }, time::{ Duration, Instant } };
use crate::{ Task, TaskLike, TaskScheduler, TaskSystemModification };



const DEFAULT_INTERVAL:Duration = Duration::from_millis(1);



pub struct TaskSystem {
	tasks:Vec<Box<dyn TaskLike>>,
	interval:Duration,

	// Running the system only once at a time and keeping a mutexed modifications queue (inside TaskScheduler) ensures 'tasks' property can be used without locking.
	run_lock:Mutex<bool>,
	task_scheduler:TaskScheduler,

	#[cfg(test)]
	pub(crate) system_loops:usize
}
impl TaskSystem {

	/* CONSTRUCTOR METHODS */

	/// Create a new TaskSystem.
	pub const fn new() -> TaskSystem {
		TaskSystem {
			tasks: Vec::new(),
			interval: DEFAULT_INTERVAL,

			run_lock: Mutex::new(false),
			task_scheduler: TaskScheduler::new(),

			#[cfg(test)]
			system_loops: usize::MAX
		}
	}



	/* MODIFICATION METHODS */

	/// Set the interval.
	pub fn set_interval(&mut self, interval:Duration) {
		self.interval = interval;
	}



	/* TASK SCHEDULING METHODS */

	/// Get a reference to the task scheduler.
	pub fn task_scheduler(&self) -> &TaskScheduler {
		&self.task_scheduler
	}

	/// Add a task to the system. Does not immediately add it, but puts a request in the queue that adds it on the first run.
	pub fn add_task<T:TaskLike + 'static>(&mut self, task:T) {
		self.task_scheduler.add_task(task);
	}

	/// Remove a task by name from the system. Does not immediately remove it, but puts a request in the queue that adds it on the first run.
	pub fn remove_task(&mut self, task_name:&str) {
		self.task_scheduler.remove_task(task_name);
	}



	/* USAGE METHODS */

	/// Run the task-system indefinitely.
	pub fn run(&mut self) {
		self.run_while(|_| true);
	}

	/// Run the system while the given statement is true.
	pub fn run_while<T>(&mut self, condition:T) where T:Fn(&TaskSystem) -> bool {
		use std::thread::sleep;

		// Get run lock.
		if !self.get_run_lock() {
			return;
		}

		// Run while condition is true.
		let mut loop_start:Instant = Instant::now();
		let mut triggered_events:Vec<String> = Vec::new();
		while condition(self) {
			let next_iteration_target:Instant = loop_start + self.interval;

			// Update tasks.
			self.inner_run_once(&loop_start, &mut triggered_events);

			// Await interval.
			let loop_end:Instant = Instant::now();
			if next_iteration_target > loop_end {
				let sleep_time:Duration = next_iteration_target - loop_end;
				loop_start = loop_end + sleep_time;
				sleep(sleep_time);
			}

			// Stop the system after a specific amount of loops in unit tests.
			#[cfg(test)]
			{
				self.system_loops -= 1;
				if self.system_loops == 0 {
					break;
				}
			}
		}

		// Release run lock.
		self.release_run_lock();
	}

	/// Get a run lock and update all tasks once.
	pub fn run_once(&mut self, now:&Instant, triggered_events:&mut Vec<String>) {
		if !self.get_run_lock() { return; }
		self.inner_run_once(now, triggered_events);
		self.release_run_lock();
	}
	
	/// Update all tasks once. Assumes the run lock has already been locked.
	fn inner_run_once(&mut self, now:&Instant, triggered_events:&mut Vec<String>) {

		// Handle modifications. Important to do before running other tasks as this could trigger events.
		for modification in self.task_scheduler.drain() {
			self.handle_modification(modification, triggered_events);
		}

		// Run all tasks.
		for task in self.tasks.iter_mut().filter(|task| task.should_run(now, triggered_events)) {
			task.run(&self.task_scheduler);
		}

		// Remove expired tasks and clear active events list.
		*triggered_events = Vec::new();
		self.tasks.retain(|task| !task.expired());
	}

	/// Run a single given task once. Does not check if the task should be ran.
	pub fn run_task_once(&mut self, task:&mut Task) {
		if !self.get_run_lock() { return; }
		task.run(&self.task_scheduler);
		self.release_run_lock();
	}

	/// Handle a single modification.
	fn handle_modification(&mut self, modification:TaskSystemModification, triggered_events:&mut Vec<String>) {
		match modification {
			TaskSystemModification::Add(task) => {
				match task.duplicate_handler() {
					crate::DuplicateHandler::KeepAll => {
						self.tasks.push(task);
					},
					crate::DuplicateHandler::KeepOld => {
						if self.tasks.iter().find(|existing_task| existing_task.name() == task.name()).is_none() {
							self.tasks.push(task);
						}
					},
					crate::DuplicateHandler::KeepNew => {
						self.handle_modification(TaskSystemModification::Remove(task.name().to_string()), triggered_events);
						self.tasks.push(task);
					}
				}
			},
			TaskSystemModification::Remove(task_name) => {
				self.tasks.retain(|task| task.name() != task_name)
			},
			TaskSystemModification::TriggerEvent(event_name) => {
				triggered_events.push(event_name);
			}
		}
	}

	/// Pause the system. Stores the current time and adds the paused time to the tasks' trigger timer upon resume.
	pub fn pause(&mut self) {
		let now:Instant = Instant::now();
		for task in &mut self.tasks {
			task.pause(&now);
		}
	}

	/// Resume the event. Adds the paused time to the tasks' trigger timer.
	pub fn resume(&mut self) {
		let now:Instant = Instant::now();
		for task in &mut self.tasks {
			task.resume(&now);
		}
	}

	/// Get a run lock.
	fn get_run_lock(&self) -> bool {
		let mut run_lock_handle:MutexGuard<'_, bool> = self.run_lock.lock().unwrap();
		if *run_lock_handle {
			eprintln!("Could not run task_system, can only run once at a time.");
			false
		} else {
			*run_lock_handle = true;
			true
		}
	}

	/// Release the run lock.
	fn release_run_lock(&self) {
		*self.run_lock.lock().unwrap() = false;
	}
}