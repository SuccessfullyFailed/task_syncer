#[cfg(test)]
mod tests {
	use std::{ thread::{ self, sleep }, time::{ Duration, Instant } };
	use crate::{ DuplicateHandler, EventSubscription, Task, TaskSystem };



	const NO_DURATION:Duration = Duration::from_millis(0);



	#[test]
	fn test_tasks_executing() {
		static mut MODIFICATION_CHECK:bool = false;

		// Create a debug task.
		let mut task_system:TaskSystem = TaskSystem::new();
		task_system.add_task(Task::new("test", |_, _| unsafe { MODIFICATION_CHECK = true; Ok(()) }));
		task_system.system_loops = 1;
		task_system.run();

		// Validate task was executed.
		assert!(unsafe { MODIFICATION_CHECK });
	}

	#[test]
	fn test_subscriptions_executing() {
		static mut MODIFICATION_CHECK:bool = false;

		// Create a debug subscription.
		let mut task_system:TaskSystem = TaskSystem::new();
		task_system.add_task(EventSubscription::new("test", "test_evt", |_| unsafe { MODIFICATION_CHECK = true; Ok(()) }));
		
		// Check no activation on incorrect triggers.
		let now:Instant = Instant::now();
		task_system.run_once(&now, &mut vec![]);
		task_system.run_once(&now, &mut vec!["test".to_string()]);
		task_system.run_once(&now, &mut vec!["test_evt_b".to_string()]);
		assert!(unsafe { !MODIFICATION_CHECK });

		// Check activation on correct trigger.
		task_system.run_once(&now, &mut vec!["a".to_string(), String::new(), "test_evt".to_string()]);
		assert!(unsafe { MODIFICATION_CHECK });
	}

	#[test]
	fn test_correct_system_loops_quantity() {
		static mut MODIFICATION_CHECK:u8 = 0;

		// Create some debug tasks.
		let mut task_system:TaskSystem = TaskSystem::new();
		task_system.add_task(Task::new("test", |_, event| unsafe { MODIFICATION_CHECK += 1; event.reschedule(NO_DURATION) }));
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
		task_system.add_task(Task::new("test", |_, event| unsafe {
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
		task_system.add_task(Task::new("test", |_, event| unsafe {
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
		task_system.add_task(Task::new("test", |_, event| unsafe {
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

	#[test]
	fn test_system_pausing() {
		static mut MODIFICATION_CHECK:u8 = 0;
		const INTERVAL:Duration = Duration::from_millis(10);

		// Create some debug tasks.
		let mut task_system:TaskSystem = TaskSystem::new();
		task_system.add_task(Task::new("test", |_, event| unsafe {
			MODIFICATION_CHECK += 1;
			if MODIFICATION_CHECK < 20 {
				event.reschedule(INTERVAL)
			} else {
				Ok(())
			}
		}));
		let end_time:Instant = Instant::now() + INTERVAL * 10;
		task_system.run_while(|_| Instant::now() < end_time);

		// Validate task was executed 10 times.
		assert_eq!(unsafe { MODIFICATION_CHECK }, 10);

		// Pause system.
		task_system.pause();
		let end_time:Instant = Instant::now() + INTERVAL * 10;
		task_system.run_while(|_| Instant::now() < end_time);
		assert_eq!(unsafe { MODIFICATION_CHECK }, 10);

		// Resume system.
		task_system.resume();
		task_system.run_once(&Instant::now(), &mut Vec::new());
		let end_time:Instant = Instant::now() + INTERVAL * 10;
		task_system.run_while(|_| Instant::now() < end_time);
		assert_eq!(unsafe { MODIFICATION_CHECK }, 20);
	}

	#[test]
	fn test_tasks_duplicate_handling() {
		static mut MODIFICATION_CHECK:u8 = 0;

		// Keep all.
		unsafe { MODIFICATION_CHECK = 0; }
		let mut task_system:TaskSystem = TaskSystem::new();
		task_system.add_task(Task::new("test", |_, _| unsafe { MODIFICATION_CHECK += 1; Ok(()) }).with_duplicate_handler(DuplicateHandler::KeepAll));
		task_system.add_task(Task::new("test", |_, _| unsafe { MODIFICATION_CHECK += 1; Ok(()) }).with_duplicate_handler(DuplicateHandler::KeepAll));
		task_system.add_task(Task::new("test", |_, _| unsafe { MODIFICATION_CHECK += 1; Ok(()) }).with_duplicate_handler(DuplicateHandler::KeepAll));
		task_system.run_once(&Instant::now(), &mut Vec::new());
		assert_eq!(unsafe { MODIFICATION_CHECK }, 3);

		// Keep old.
		unsafe { MODIFICATION_CHECK = 0; }
		let mut task_system:TaskSystem = TaskSystem::new();
		task_system.add_task(Task::new("test", |_, _| unsafe { MODIFICATION_CHECK = 1; Ok(()) }).with_duplicate_handler(DuplicateHandler::KeepOld));
		task_system.add_task(Task::new("test", |_, _| unsafe { MODIFICATION_CHECK = 2; Ok(()) }).with_duplicate_handler(DuplicateHandler::KeepOld));
		task_system.add_task(Task::new("test", |_, _| unsafe { MODIFICATION_CHECK = 3; Ok(()) }).with_duplicate_handler(DuplicateHandler::KeepOld));
		task_system.run_once(&Instant::now(), &mut Vec::new());
		assert_eq!(unsafe { MODIFICATION_CHECK }, 1);

		// Keep new.
		unsafe { MODIFICATION_CHECK = 0; }
		let mut task_system:TaskSystem = TaskSystem::new();
		task_system.add_task(Task::new("test", |_, _| unsafe { MODIFICATION_CHECK = 1; Ok(()) }).with_duplicate_handler(DuplicateHandler::KeepNew));
		task_system.add_task(Task::new("test", |_, _| unsafe { MODIFICATION_CHECK = 2; Ok(()) }).with_duplicate_handler(DuplicateHandler::KeepNew));
		task_system.add_task(Task::new("test", |_, _| unsafe { MODIFICATION_CHECK = 3; Ok(()) }).with_duplicate_handler(DuplicateHandler::KeepNew));
		task_system.run_once(&Instant::now(), &mut Vec::new());
		assert_eq!(unsafe { MODIFICATION_CHECK }, 3);

		// Mixed.
		unsafe { MODIFICATION_CHECK = 0; }
		let mut task_system:TaskSystem = TaskSystem::new();
		task_system.add_task(Task::new("test", |_, _| unsafe { MODIFICATION_CHECK += 1; Ok(()) }).with_duplicate_handler(DuplicateHandler::KeepOld));
		task_system.add_task(Task::new("test", |_, _| unsafe { MODIFICATION_CHECK += 2; Ok(()) }).with_duplicate_handler(DuplicateHandler::KeepNew));
		task_system.add_task(Task::new("test", |_, _| unsafe { MODIFICATION_CHECK += 3; Ok(()) }).with_duplicate_handler(DuplicateHandler::KeepAll));
		task_system.run_once(&Instant::now(), &mut Vec::new());
		assert_eq!(unsafe { MODIFICATION_CHECK }, 5);
	}

	#[test]
	#[allow(static_mut_refs)]
	fn test_cannot_run_twice() {
		static mut SYSTEM:TaskSystem = TaskSystem::new();
		static mut MODIFICATION_CHECK:u8 = 0;
		static mut TIMER:u8 = 0;
		
		unsafe {
			SYSTEM.add_task(Task::new("test", |_, _| { MODIFICATION_CHECK += 1; Ok(()) }));

			// Launch the task in a bunch of threads.
			for _ in 0..10 {
				thread::spawn(|| SYSTEM.run_once(&Instant::now(), &mut Vec::new()));
				thread::spawn(||
					SYSTEM.run_while(|_| {
						if TIMER < 100 {
							TIMER += 1;
							true
						} else {
							false
						}
					})
				);
			}

			// Make sure that after running all threads, the event has only run once.
			sleep(Duration::from_millis(100));
			assert_eq!(MODIFICATION_CHECK, 1);
		}
	}

	#[test]
	fn test_run_single_task() {
		static mut MODIFICATION_CHECK:u8 = 0;

		// Create some debug tasks.
		let mut task_system:TaskSystem = TaskSystem::new();
		task_system.run_task_once(&mut Task::new("test", |_, _| unsafe { MODIFICATION_CHECK = 1; Ok(()) }));
		task_system.system_loops = 1;
		task_system.run();

		// Validate task was executed.
		assert_eq!(unsafe { MODIFICATION_CHECK }, 1);
	}

	#[test]
	fn test_single_task_can_mod_scheduler() {
		static mut MODIFICATION_CHECK:u8 = 0;

		// Create some debug tasks.
		let mut task_system:TaskSystem = TaskSystem::new();
		task_system.run_task_once(&mut Task::new("test", |scheduler, _| {
			scheduler.add_task(Task::new("sub_1", |_, _| unsafe { MODIFICATION_CHECK = 1; Ok(()) }));
			scheduler.add_task(Task::new("sub_2", |_, _| unsafe { MODIFICATION_CHECK = 2; Ok(()) }));
			scheduler.remove_task("sub_2");
			Ok(())
		}));
		task_system.system_loops = 3;
		task_system.run();

		// Validate task was executed.
		assert_eq!(unsafe { MODIFICATION_CHECK }, 1);
	}
}