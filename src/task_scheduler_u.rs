#[cfg(test)]
mod tests {
	use crate::{ EventSubscription, Task, TaskSystem };



	#[test]
	fn test_tasks_using_task_scheduler_execute() {
		static mut MODIFICATION_CHECK:u8 = 0;

		// Create some debug tasks.
		let mut task_system:TaskSystem = TaskSystem::new();
		task_system.add_task(Task::new("insert_task_mods", |scheduler, _| {
			scheduler.add_task(Task::new("run_a", |_, _| unsafe { MODIFICATION_CHECK |= 1; Ok(()) }));
			scheduler.add_task(Task::new("run_b", |_, _| unsafe { MODIFICATION_CHECK |= 2; Ok(()) }));
			scheduler.add_task(Task::new("run_c", |_, _| unsafe { MODIFICATION_CHECK |= 4; Ok(()) }));
			scheduler.add_task(EventSubscription::new("run_d", "test_evt_a", |_| unsafe { MODIFICATION_CHECK |= 8; Ok(()) }));
			scheduler.add_task(EventSubscription::new("run_e", "test_evt_b", |_| unsafe { MODIFICATION_CHECK |= 16; Ok(()) }));
			scheduler.add_task(EventSubscription::new("run_f", "test_evt_c", |_| unsafe { MODIFICATION_CHECK |= 32; Ok(()) }));
			scheduler.add_task(Task::new("trigger_event", |scheduler, _| { scheduler.trigger_event("test_evt_b"); Ok(()) }));
			scheduler.remove_task("run_c");
			scheduler.remove_task("run_f");
			Ok(())
		}));
		task_system.system_loops = 3;
		task_system.run();

		// Validate task was executed.
		const EXPECTED_VALUES:[u8; 3] = [1, 2, 16];
		let mut actual_value:u8 = unsafe { MODIFICATION_CHECK };
		for expected_value in EXPECTED_VALUES {
			assert_eq!(actual_value & expected_value, expected_value);
			actual_value &= !expected_value;
		}
		assert_eq!(actual_value, 0);
	}
}