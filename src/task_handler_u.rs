#[cfg(test)]
mod tests {
	use std::{ sync::Mutex, time::Instant };
	use crate::{ Task, TaskEvent, TaskHandler };



	#[test]
	fn handler_none() {
		static RUN_PROOF:Mutex<u8> = Mutex::new(0);

		let mut handler:TaskHandler = TaskHandler::None;
		let mut event:TaskEvent = TaskEvent::default();
		
		let now:Instant = Instant::now();
		for _ in 0..64 {
			handler.run(&now, &mut event).unwrap();
			assert_eq!(*RUN_PROOF.lock().unwrap(), 0);
			assert_eq!(event.expired, true);
		}
	}

	#[test]
	fn handler_fn() {
		static RUN_PROOF:Mutex<u8> = Mutex::new(0);

		let mut handler:TaskHandler = TaskHandler::Fn(Box::new(|event| {
			*RUN_PROOF.lock().unwrap() += 1;
			if *RUN_PROOF.lock().unwrap() == 50 {
				event.expire();
			}
			Ok(())
		}));
		let mut event:TaskEvent = TaskEvent::default();
		
		let now:Instant = Instant::now();
		for index in 1..=64 {
			handler.run(&now, &mut event).unwrap();
			assert_eq!(*RUN_PROOF.lock().unwrap(), index);
			assert_eq!(event.expired, index >= 50);
		}
	}

	#[test]
	fn handler_fn_mut() {
		static RUN_PROOF:Mutex<u8> = Mutex::new(0);

		let mut owned_index:u8 = 0;
		let mut handler:TaskHandler = TaskHandler::Fn(Box::new(move |event| {
			*RUN_PROOF.lock().unwrap() += 1;
			owned_index += 1;
			if owned_index == 50 {
				event.expire();
			}
			Ok(())
		}));
		let mut event:TaskEvent = TaskEvent::default();
		
		let now:Instant = Instant::now();
		for index in 1..=64 {
			handler.run(&now, &mut event).unwrap();
			assert_eq!(*RUN_PROOF.lock().unwrap(), index);
			assert_eq!(event.expired, index >= 50);
		}
	}

	#[test]
	fn handler_task() {
		static RUN_PROOF:Mutex<u8> = Mutex::new(0);

		let mut handler:TaskHandler = TaskHandler::Task(
			Task::new("test_task", |event:&mut TaskEvent| {
				if *RUN_PROOF.lock().unwrap() < 50 {
					*RUN_PROOF.lock().unwrap() += 1;
					event.repeated()
				} else {
					Ok(())
				}
			})
		);
		let mut event:TaskEvent = TaskEvent::default();
		
		let now:Instant = Instant::now();
		for index in 1..=64 {
			handler.run(&now, &mut event).unwrap();
			assert_eq!(*RUN_PROOF.lock().unwrap(), index.min(50));
			assert_eq!(event.expired, index > 50);
		}
	}

	#[test]
	fn handler_repeat() {
		static RUN_PROOF:Mutex<u8> = Mutex::new(0);

		let mut handler:TaskHandler = TaskHandler::Repeat((
			Box::new(TaskHandler::Fn(Box::new(|_event| {
				*RUN_PROOF.lock().unwrap() += 1;
				Ok(())
			}))),
			0..10
		));
		let mut event:TaskEvent = TaskEvent::default();
		
		let now:Instant = Instant::now();
		for index in 1..=64 {
			handler.run(&now, &mut event).unwrap();
			assert_eq!(*RUN_PROOF.lock().unwrap(), index.min(10));
			assert_eq!(event.expired, index >= 10);
		}
	}

	#[test]
	fn handler_list() {
		static RUN_PROOF:Mutex<u8> = Mutex::new(0);

		let mut handler:TaskHandler = TaskHandler::List((
			vec![
				TaskHandler::Fn(Box::new(|event| {
					*RUN_PROOF.lock().unwrap() += 1;
					if *RUN_PROOF.lock().unwrap() == 10 {
						event.expire();
					}
					Ok(())
				})),
				TaskHandler::Fn(Box::new(|event| {
					*RUN_PROOF.lock().unwrap() += 2;
					if *RUN_PROOF.lock().unwrap() == 30 {
						event.expire();
					}
					Ok(())
				})),
				TaskHandler::Fn(Box::new(|event| {
					*RUN_PROOF.lock().unwrap() += 3;
					if *RUN_PROOF.lock().unwrap() == 60 {
						event.expire();
					}
					Ok(())
				}))
			],
			0
		));
		let mut event:TaskEvent = TaskEvent::default();
		
		let now:Instant = Instant::now();
		for index in 1..=64 {
			handler.run(&now, &mut event).unwrap();
			assert_eq!(*RUN_PROOF.lock().unwrap(), if index < 10 { index } else if index < 20 { 10 + (index - 10) * 2 } else if index < 30 { 30 + (index - 20) * 3 } else { 60 });
			assert_eq!(event.expired, index >= 30);
		}
	}
}