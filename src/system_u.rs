#[cfg(test)]
mod test {
	use std::{ sync::Mutex, thread::sleep, time::Duration };
	use crate::{ Task, TaskEvent, TaskSystem };


	#[test]
	fn test_singular_system_cycle() {
		static RUN_PROOF:Mutex<usize> = Mutex::new(0);

		// Create system with a task.
		let mut system:TaskSystem = TaskSystem::new();
		system.add_task(Task::new("task1", |event:&mut TaskEvent| {
			*RUN_PROOF.lock().unwrap() += 1;
			event.reschedule_r(Duration::from_millis(10))
		}));

		// Run once.
		system.start();
		sleep(Duration::from_millis(6));
		system.stop();

		// Assert results.
		assert_eq!(*RUN_PROOF.lock().unwrap(), 1);
	}

	#[test]
	fn test_multiple_system_cycles() {
		static RUN_PROOF_A:Mutex<usize> = Mutex::new(0);
		static RUN_PROOF_B:Mutex<usize> = Mutex::new(0);

		// Create system with two tasks.
		let mut system:TaskSystem = TaskSystem::new();
		system.add_task(Task::new("task1", |event:&mut TaskEvent| {
			*RUN_PROOF_A.lock().unwrap() += 1;
			event.reschedule_r(Duration::from_millis(10))
		}));
		system.add_task(Task::new("task2", |_event:&mut TaskEvent| {
			*RUN_PROOF_B.lock().unwrap() += 1;
			Ok(())
		}));

		// Run for a little over 5 loops of the first tasks.
		system.start();
		sleep(Duration::from_millis(45));
		system.stop();

		// Assert results.
		assert_eq!(*RUN_PROOF_A.lock().unwrap(), 5);
		assert_eq!(*RUN_PROOF_B.lock().unwrap(), 1);
	}

	#[test]
	fn test_run_async() {
		static RUN_PROOF_A:Mutex<usize> = Mutex::new(0);
		static RUN_PROOF_B:Mutex<usize> = Mutex::new(0);
		static RUN_PROOF_C:Mutex<usize> = Mutex::new(0);

		// Create system with two tasks.
		let mut system:TaskSystem = TaskSystem::new();
		system.add_task(Task::new("task1", |event:&mut TaskEvent| {
			*RUN_PROOF_A.lock().unwrap() += 1;
			event.reschedule_r(Duration::from_millis(10))
		}));
		system.add_task(Task::new("task2", |_event:&mut TaskEvent| {
			*RUN_PROOF_B.lock().unwrap() += 1;
			Ok(())
		}));

		// Start system and add a third task while still running.
		system.start();
		sleep(Duration::from_millis(22));
		system.add_task(Task::new("task3", |event:&mut TaskEvent| {
			*RUN_PROOF_C.lock().unwrap() += 1;
			event.reschedule_r(Duration::from_millis(10))
		}));
		sleep(Duration::from_millis(22));

		// Stop the system.
		system.stop();
		sleep(Duration::from_millis(20));

		// Assert results.
		assert_eq!(*RUN_PROOF_A.lock().unwrap(), 5);
		assert_eq!(*RUN_PROOF_B.lock().unwrap(), 1);
		assert_eq!(*RUN_PROOF_C.lock().unwrap(), 3);
	}

	#[test]
	fn test_system_keeps_tasks_after_stopping() {
		static RUN_PROOF_A:Mutex<usize> = Mutex::new(0);
		static RUN_PROOF_B:Mutex<usize> = Mutex::new(0);

		// Create system with two tasks.
		let mut system:TaskSystem = TaskSystem::new();
		system.add_task(Task::new("task1", |event:&mut TaskEvent| {
			*RUN_PROOF_A.lock().unwrap() += 1;
			event.reschedule_r(Duration::from_millis(10))
		}));
		system.add_task(Task::new("task2", |_event:&mut TaskEvent| {
			*RUN_PROOF_B.lock().unwrap() += 1;
			Ok(())
		}));

		// Run the system twice, confirming the system keeps the repeating task.
		system.start();
		sleep(Duration::from_millis(15));
		system.stop();

		// Assert results.
		assert_eq!(*RUN_PROOF_A.lock().unwrap(), 2);
		assert_eq!(*RUN_PROOF_B.lock().unwrap(), 1);
	}
}