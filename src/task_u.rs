#[cfg(test)]
mod tests {
	use std::{ sync::Mutex, time::{Duration, Instant} };
	use crate::{ Task, TaskEvent };



	#[test]
	fn test_task_handler_runs_handler() {
		static RUN_PROOF:Mutex<u8> = Mutex::new(0);

		let mut task:Task = Task::new("test_task", |event:&mut TaskEvent| {
			*RUN_PROOF.lock().unwrap() += 1;
			if *RUN_PROOF.lock().unwrap() < 50 {
				event.repeated()
			} else {
				Ok(())
			}
		});
		
		let now:Instant = Instant::now();
		for index in 1..=64 {
			task.run(&now).unwrap();
			assert_eq!(*RUN_PROOF.lock().unwrap(), index.min(50));
			assert_eq!(task.event.expired, index >= 50);
		}
	}

	#[test]
	fn test_task_handler_delay_triggers() {
		static RUN_PROOF:Mutex<u8> = Mutex::new(0);

		let mut task:Task = Task::new("test_task", |event:&mut TaskEvent| {
			*RUN_PROOF.lock().unwrap() += 1;
			event.rescheduled(Duration::from_millis(100))
		});
		
		let mut now:Instant = Instant::now();
		for index in 0..64 {
			task.run(&now).unwrap();
			assert_eq!(task.event.expired, false);
			assert_eq!(*RUN_PROOF.lock().unwrap(), (index / 5) + 1);
			now += Duration::from_millis(20);
		}
	}
}