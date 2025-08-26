use std::time::{ Duration, Instant };
use crate::Task;



const DEFAULT_INTERVAL:Duration = Duration::from_millis(1);



pub struct TaskSystem {
	tasks:Vec<Task>,
	interval:Duration,

	#[cfg(test)]
	pub(crate) system_loops:usize
}
impl TaskSystem {

	/* CONSTRUCTOR METHODS */

	/// Create a new TaskSystem.
	pub fn new() -> TaskSystem {
		TaskSystem {
			tasks: Vec::new(),
			interval: DEFAULT_INTERVAL,

			#[cfg(test)]
			system_loops: usize::MAX
		}
	}



	/* MODIFICATION METHODS */

	/// Set the interval.
	pub fn set_interval(&mut self, interval:Duration) {
		self.interval = interval;
	}



	/* TASK METHODS */

	/// Add a task to the system.
	pub fn add_task(&mut self, task:Task) {
		self.tasks.push(task);
	}



	/* USAGE METHODS */

	/// Run the task-system indefinitely.
	pub fn run(&mut self) {
		self.run_while(|_| true);
	}

	/// Run the system while the given statement is true.
	pub fn run_while<T>(&mut self, a:T) where T:Fn(&TaskSystem) -> bool {
		use std::thread::sleep;

		let mut loop_start:Instant = Instant::now();
		while a(self) {
			let next_iteration_target:Instant = loop_start + self.interval;

			// Update tasks.
			self.run_once(&loop_start);

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
	}

	/// Update all tasks once.
	pub fn run_once(&mut self, now:&Instant) {
		for task in self.tasks.iter_mut().filter(|task| task.should_run(now)) {
			task.run();
		}
		self.tasks.retain(|task| !task.expired());
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
		for task in &mut self.tasks {
			task.resume();
		}
	}
}