#[cfg(test)]
mod tests {
	use crate::{ Task, TaskSystem };



	#[test]
	fn test_tasks_using_task_scheduler_execute() {
		static mut MODIFICATION_CHECK:u8 = 0;

		// Create some debug tasks.
		let mut task_system:TaskSystem = TaskSystem::new();
		task_system.add_task(Task::new("insert_task_mods", |scheduler, _| {
			scheduler.add_task(Task::new("run_a", |_, _| unsafe { MODIFICATION_CHECK = 1; Ok(()) }));
			scheduler.add_task(Task::new("run_b", |_, _| unsafe { MODIFICATION_CHECK = 2; Ok(()) }));
			scheduler.add_task(Task::new("run_c", |_, _| unsafe { MODIFICATION_CHECK = 3; Ok(()) }));
			scheduler.remove_task("run_c");
			Ok(())
		}));
		task_system.system_loops = 10;
		task_system.run();

		// Validate task was executed.
		assert_eq!(unsafe { MODIFICATION_CHECK }, 2);
	}
}