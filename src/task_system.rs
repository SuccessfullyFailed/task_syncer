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
		use std::thread::sleep;

		loop {
			let loop_start:Instant = Instant::now();
			let next_iteration_target:Instant = loop_start + self.interval;

			// Update tasks.
			for task in self.tasks.iter_mut().filter(|task| task.should_run(&loop_start)) {
				task.run();
			}
			self.tasks.retain(|task| !task.expired());

			// Await interval.
			let loop_end:Instant = Instant::now();
			if next_iteration_target > loop_end {
				sleep(next_iteration_target - loop_end);
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
}