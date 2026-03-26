#[cfg(test)]
mod tests {
	use crate::{ Task, TaskEvent, TaskModificationRequest, TaskScheduler };


	
	#[test]
	fn test_add_and_drain_task() {
		let scheduler:TaskScheduler = TaskScheduler::default();
		scheduler.add_task(Task::new("task1", |_:&mut TaskEvent| Ok(())));
		scheduler.add_task(Task::new("task2", |_:&mut TaskEvent| Ok(())));

		let mut drained:Vec<TaskModificationRequest> = scheduler.drain();
		match drained.remove(0) {
			TaskModificationRequest::AddTask(task) => assert_eq!(task.name, "task1"),
			_ => panic!("First drained was not a task-add request.")
		}
		match drained.remove(0) {
			TaskModificationRequest::AddTask(task) => assert_eq!(task.name, "task2"),
			_ => panic!("First drained was not a task-add request.")
		}
		assert_eq!(scheduler.drain().len(), 0);
	}
}