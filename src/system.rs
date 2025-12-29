use std::{ error::Error, sync::Mutex, thread::sleep, time::{ Duration, Instant } };
use crate::Task;



const MAX_INTERVAL:Duration = Duration::from_millis(250);



pub struct TaskSystem {
	tasks:Vec<Task>, // Tasks are only modified in the running method, so this does not need any synchronization technique.
	scheduler:TaskScheduler,
	error_handler:Box<dyn Fn(&str, Box<dyn Error>)>
}
impl TaskSystem {

	/* CONSTRUCTOR METHODS */

	/// Create a new system.
	pub fn new() -> TaskSystem {
		TaskSystem::default()
	}



	/* RUNNING METHODS */

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
		for modification in self.scheduler.0.lock().unwrap().drain(..) {
			match modification {
				TaskModificationRequest::AddTask(task) => self.tasks.push(task),
				TaskModificationRequest::RetainTasks(filter) => self.tasks.retain(filter)
			}
		}
	}
}
impl Default for TaskSystem {
	fn default() -> Self {
		TaskSystem {
			tasks: Vec::new(),
			scheduler: TaskScheduler::default(),
			error_handler: Box::new(|task_name, error| eprintln!("task '{task_name}' returned error: '{error}'"))
		}
	}
}



pub enum TaskModificationRequest { AddTask(Task), RetainTasks(Box<dyn FnMut(&Task) -> bool>) }
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
}
impl Default for TaskScheduler {
	fn default() -> Self {
		TaskScheduler(Mutex::new(Vec::new()))
	}
}