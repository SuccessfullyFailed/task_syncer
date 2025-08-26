#[cfg(test)]
mod test {
	use std::{ thread::sleep, time::{ Duration, Instant } };
	use crate::Task;



	#[test]
	fn test_main_handler_execute() {
		static mut MODIFICATION_CHECK:bool = false;
		let mut task:Task = Task::new("test", |_| unsafe { MODIFICATION_CHECK = true; Ok(()) });
		task.run();
		assert!(unsafe { MODIFICATION_CHECK });
	}

	#[test]
	fn test_error_handler_execute() {
		static mut MODIFICATION_CHECK:u8 = 0;
		let mut task:Task = Task::new("test", |_| unsafe { MODIFICATION_CHECK = 1; Err("".into()) }).on_error(|_, _| unsafe { MODIFICATION_CHECK = 2; });
		task.run();
		assert_eq!(unsafe { MODIFICATION_CHECK }, 2);
	}

	#[test]
	fn test_then_handlers_execute() {
		static mut MODIFICATION_CHECK:u8 = 0;
		let mut task:Task = Task::new("test", |_| unsafe { MODIFICATION_CHECK = 1; Ok(()) }).then(|_| unsafe { MODIFICATION_CHECK = 2;  Ok(()) }).then(|_| unsafe { MODIFICATION_CHECK = 4;  Ok(()) });
		task.run();
		assert_eq!(unsafe { MODIFICATION_CHECK }, 1);
		task.run();
		assert_eq!(unsafe { MODIFICATION_CHECK }, 2);
		task.run();
		assert_eq!(unsafe { MODIFICATION_CHECK }, 4);
	}

	#[test]
	fn test_expiration() {
		let mut task:Task = Task::new("test", |_| Ok(())).then(|_| Ok(()));
		task.run();
		assert!(!task.expired());
		task.run();
		assert!(task.expired());
	}

	#[test]
	fn test_scheduling_timer() {
		let now:Instant = Instant::now();
		
		let task_a:Task = Task::new("test", |_| Ok(()));
		let task_b:Task = Task::new("test", |_| Ok(())).delay(Duration::from_millis(10));

		assert!(!task_a.should_run(&now));
		assert!(task_a.should_run(&(now + Duration::from_millis(1))));
		assert!(task_a.should_run(&(now + Duration::from_millis(11))));
		assert!(!task_b.should_run(&now));
		assert!(!task_b.should_run(&(now + Duration::from_millis(1))));
		assert!(task_b.should_run(&(now + Duration::from_millis(11))));
	}

	#[test]
	fn test_task_pausing() {
		let now:Instant = Instant::now();
		
		let mut task:Task = Task::new("test", |_| Ok(())).delay(Duration::from_millis(3));

		assert!(!task.should_run(&now));
		assert!(!task.should_run(&(now + Duration::from_millis(1))));
		assert!(task.should_run(&(now + Duration::from_millis(4))));
		
		let pausing_time:Duration = Duration::from_millis(250);
		task.pause(&now);
		sleep(pausing_time);
		task.resume();

		assert!(!task.should_run(&now));
		assert!(!task.should_run(&(now + Duration::from_millis(1))));
		assert!(!task.should_run(&(now + Duration::from_millis(4))));
		assert!(!task.should_run(&(now + Duration::from_millis(250))));
		assert!(task.should_run(&(now + Duration::from_millis(260))));
	}
}