#[cfg(test)]
mod tests {
	use std::time::{Duration, Instant};
	use crate::{ Task, TaskSystem };



	const NO_DURATION:Duration = Duration::from_millis(0);




	#[test]
	fn test_tasks_executing() {
		static mut MODIFICATION_CHECK:bool = false;

		// Create some debug tasks.
		let mut task_system:TaskSystem = TaskSystem::new();
		task_system.add_task(Task::new("test", |_| unsafe { MODIFICATION_CHECK = true; Ok(()) }));
		task_system.system_loops = 1;
		task_system.run();

		// Validate task was executed.
		assert!(unsafe { MODIFICATION_CHECK });
	}

	#[test]
	fn test_correct_system_loops_quantity() {
		static mut MODIFICATION_CHECK:u8 = 0;

		// Create some debug tasks.
		let mut task_system:TaskSystem = TaskSystem::new();
		task_system.add_task(Task::new("test", |event| unsafe { MODIFICATION_CHECK += 1; event.reschedule(NO_DURATION) }));
		task_system.system_loops = 20;
		task_system.run();

		// Validate task was executed.
		assert_eq!(unsafe { MODIFICATION_CHECK }, 20);
	}

	#[test]
	fn test_event_rescheduling() {
		static mut MODIFICATION_CHECK:u8 = 0;

		// Create some debug tasks.
		let mut task_system:TaskSystem = TaskSystem::new();
		task_system.add_task(Task::new("test", |event| unsafe {
			MODIFICATION_CHECK += 1;
			if MODIFICATION_CHECK < 20 {
				event.reschedule(NO_DURATION)
			} else {
				Ok(())
			}
		}));
		task_system.system_loops = 20;
		task_system.run();

		// Validate task was executed.
		assert_eq!(unsafe { MODIFICATION_CHECK }, 20);
	}

	#[test]
	fn test_loop_waits_before_next_loop() {
		static mut MODIFICATION_CHECK:u8 = 0;

		// Create some debug tasks.
		let mut task_system:TaskSystem = TaskSystem::new();
		task_system.add_task(Task::new("test", |event| unsafe {
			MODIFICATION_CHECK += 1;
			if MODIFICATION_CHECK < 20 {
				event.reschedule(NO_DURATION)
			} else {
				Ok(())
			}
		}));
		task_system.system_loops = 20;

		// Run timed task system.
		let start:Instant = Instant::now();
		task_system.run();
		let duration:Duration = start.elapsed();

		// Validate task was executed.
		assert!(duration.as_millis() > 15);
	}

	#[test]
	fn test_loops_run_efficiently() {
		static mut MODIFICATION_CHECK:u32 = 0;

		// Create some debug tasks.
		let mut task_system:TaskSystem = TaskSystem::new();
		task_system.add_task(Task::new("test", |event| unsafe {
			MODIFICATION_CHECK += 1;
			event.reschedule(NO_DURATION)
		}));
		task_system.set_interval(NO_DURATION);
		task_system.system_loops = 1_000_000;

		// Run timed task system.
		let start:Instant = Instant::now();
		task_system.run();
		let duration:Duration = start.elapsed();

		// Validate task was executed.
		println!("Took {}ms to do 1_000_000 loops.", duration.as_millis()); // 75ms in release mode.
		assert!(duration.as_millis() < 1000);
	}
}