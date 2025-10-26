#[cfg(test)]
mod test {
	use crate::{ EventSubscription, TaskLike, TaskScheduler };
	use std::time::Instant;



	#[test]
	fn test_run_on_event_trigger() {
		static mut MODIFICATION_CHECK:bool = false;
		let mut task:EventSubscription = EventSubscription::new("test", "test_evt", |_| unsafe { MODIFICATION_CHECK = true; Ok(()) });
		assert!(!task.should_run(&Instant::now(), &[]));
		assert!(!task.should_run(&Instant::now(), &["test".to_string()]));
		assert!(!task.should_run(&Instant::now(), &["test_evt_b".to_string()]));
		assert!(task.should_run(&Instant::now(), &["test_evt".to_string()]));
		assert!(task.should_run(&Instant::now(), &["a".to_string(), String::new(), "test_evt".to_string()]));
		task.run(&TaskScheduler::new());
		assert!(unsafe { MODIFICATION_CHECK });
	}

	#[test]
	fn test_main_handler_execute() {
		static mut MODIFICATION_CHECK:bool = false;
		let mut task:EventSubscription = EventSubscription::new("test", "test_evt", |_| unsafe { MODIFICATION_CHECK = true; Ok(()) });
		task.run(&TaskScheduler::new());
		assert!(unsafe { MODIFICATION_CHECK });
	}

	#[test]
	fn test_error_handler_execute() {
		static mut MODIFICATION_CHECK:u8 = 0;
		let mut task:EventSubscription = EventSubscription::new("test", "test_evt", |_| unsafe { MODIFICATION_CHECK = 1; Err("".into()) }).catch(|_, _| unsafe { MODIFICATION_CHECK = 2; });
		task.run(&TaskScheduler::new());
		assert_eq!(unsafe { MODIFICATION_CHECK }, 2);
	}

	#[test]
	fn test_expiration() {
		let mut task:EventSubscription = EventSubscription::new("test", "test_evt", |_| Ok(()));
		task.run(&TaskScheduler::new());
		assert!(!task.expired());
		task.run(&TaskScheduler::new());
		assert!(!task.expired());
	}
}