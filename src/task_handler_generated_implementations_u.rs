#[cfg(test)]
mod tests {
	use crate::{ TaskHandlerSource, Event, TaskHandler };
	use std::{ error::Error, sync::{ Arc, Mutex } };


	#[test]
	fn handler_source_box_fnmut_0args_no_result_specified_source() {
		static RUN_PROOF:Mutex<u8> = Mutex::new(0);
		
		let mut owned_index:u8 = 0;
		let handler_source:Box<dyn FnMut()  + Send + Sync + 'static> = Box::new(
			move || {
				owned_index += 1;
				*RUN_PROOF.lock().unwrap() = owned_index;
			}
		);
		let mut handler:TaskHandler = handler_source.into_handler();
		let mut event:Event = Event::default();
		
		for index in 1..=64 {
			handler.run(&mut event).unwrap();
			assert_eq!(*RUN_PROOF.lock().unwrap(), index);
			assert_eq!(event.expired, false);
		}
	}
	
	#[test]
	fn handler_source_box_fnmut_0args_has_result_specified_source() {
		static RUN_PROOF:Mutex<u8> = Mutex::new(0);
		
		let mut owned_index:u8 = 0;
		let handler_source:Box<dyn FnMut() -> Result<(), Box<dyn Error>> + Send + Sync + 'static> = Box::new(
			move || {
				owned_index += 1;
				*RUN_PROOF.lock().unwrap() = owned_index;
				Ok(())
			}
		);
		let mut handler:TaskHandler = handler_source.into_handler();
		let mut event:Event = Event::default();
		
		for index in 1..=64 {
			handler.run(&mut event).unwrap();
			assert_eq!(*RUN_PROOF.lock().unwrap(), index);
			assert_eq!(event.expired, false);
		}
	}
	
	#[test]
	fn handler_source_box_fnmut_1args_no_result_specified_source() {
		static RUN_PROOF:Mutex<u8> = Mutex::new(0);
		
		let mut owned_index:u8 = 0;
		let handler_source:Box<dyn FnMut(&mut Event)  + Send + Sync + 'static> = Box::new(
			move |event| {
				owned_index += 1;
				*RUN_PROOF.lock().unwrap() = owned_index;
				if *RUN_PROOF.lock().unwrap() == 50 {{ event.expire(); }}
			}
		);
		let mut handler:TaskHandler = handler_source.into_handler();
		let mut event:Event = Event::default();
		
		for index in 1..=64 {
			handler.run(&mut event).unwrap();
			assert_eq!(*RUN_PROOF.lock().unwrap(), index);
			assert_eq!(event.expired, index >= 50);
		}
	}
	
	#[test]
	fn handler_source_box_fnmut_1args_has_result_specified_source() {
		static RUN_PROOF:Mutex<u8> = Mutex::new(0);
		
		let mut owned_index:u8 = 0;
		let handler_source:Box<dyn FnMut(&mut Event) -> Result<(), Box<dyn Error>> + Send + Sync + 'static> = Box::new(
			move |event| {
				owned_index += 1;
				*RUN_PROOF.lock().unwrap() = owned_index;
				if *RUN_PROOF.lock().unwrap() == 50 {{ event.expire(); }}
				Ok(())
			}
		);
		let mut handler:TaskHandler = handler_source.into_handler();
		let mut event:Event = Event::default();
		
		for index in 1..=64 {
			handler.run(&mut event).unwrap();
			assert_eq!(*RUN_PROOF.lock().unwrap(), index);
			assert_eq!(event.expired, index >= 50);
		}
	}
	
	#[test]
	fn handler_source_box_fn_0args_no_result_specified_source() {
		static RUN_PROOF:Mutex<u8> = Mutex::new(0);
		
		let handler_source:Box<dyn Fn()  + Send + Sync + 'static> = Box::new(
			|| {
				*RUN_PROOF.lock().unwrap() += 1;
			}
		);
		let mut handler:TaskHandler = handler_source.into_handler();
		let mut event:Event = Event::default();
		
		for index in 1..=64 {
			handler.run(&mut event).unwrap();
			assert_eq!(*RUN_PROOF.lock().unwrap(), index);
			assert_eq!(event.expired, false);
		}
	}
	
	#[test]
	fn handler_source_box_fn_0args_has_result_specified_source() {
		static RUN_PROOF:Mutex<u8> = Mutex::new(0);
		
		let handler_source:Box<dyn Fn() -> Result<(), Box<dyn Error>> + Send + Sync + 'static> = Box::new(
			|| {
				*RUN_PROOF.lock().unwrap() += 1;
				Ok(())
			}
		);
		let mut handler:TaskHandler = handler_source.into_handler();
		let mut event:Event = Event::default();
		
		for index in 1..=64 {
			handler.run(&mut event).unwrap();
			assert_eq!(*RUN_PROOF.lock().unwrap(), index);
			assert_eq!(event.expired, false);
		}
	}
	
	#[test]
	fn handler_source_box_fn_1args_no_result_specified_source() {
		static RUN_PROOF:Mutex<u8> = Mutex::new(0);
		
		let handler_source:Box<dyn Fn(&mut Event)  + Send + Sync + 'static> = Box::new(
			|event| {
				*RUN_PROOF.lock().unwrap() += 1;
				if *RUN_PROOF.lock().unwrap() == 50 {{ event.expire(); }}
			}
		);
		let mut handler:TaskHandler = handler_source.into_handler();
		let mut event:Event = Event::default();
		
		for index in 1..=64 {
			handler.run(&mut event).unwrap();
			assert_eq!(*RUN_PROOF.lock().unwrap(), index);
			assert_eq!(event.expired, index >= 50);
		}
	}
	
	#[test]
	fn handler_source_box_fn_1args_has_result_specified_source() {
		static RUN_PROOF:Mutex<u8> = Mutex::new(0);
		
		let handler_source:Box<dyn Fn(&mut Event) -> Result<(), Box<dyn Error>> + Send + Sync + 'static> = Box::new(
			|event| {
				*RUN_PROOF.lock().unwrap() += 1;
				if *RUN_PROOF.lock().unwrap() == 50 {{ event.expire(); }}
				Ok(())
			}
		);
		let mut handler:TaskHandler = handler_source.into_handler();
		let mut event:Event = Event::default();
		
		for index in 1..=64 {
			handler.run(&mut event).unwrap();
			assert_eq!(*RUN_PROOF.lock().unwrap(), index);
			assert_eq!(event.expired, index >= 50);
		}
	}
	
	#[test]
	fn handler_source_arc_fn_0args_no_result_specified_source() {
		static RUN_PROOF:Mutex<u8> = Mutex::new(0);
		
		let handler_source:Arc<dyn Fn()  + Send + Sync + 'static> = Arc::new(
			|| {
				*RUN_PROOF.lock().unwrap() += 1;
			}
		);
		let mut handler:TaskHandler = handler_source.into_handler();
		let mut event:Event = Event::default();
		
		for index in 1..=64 {
			handler.run(&mut event).unwrap();
			assert_eq!(*RUN_PROOF.lock().unwrap(), index);
			assert_eq!(event.expired, false);
		}
	}
	
	#[test]
	fn handler_source_arc_fn_0args_has_result_specified_source() {
		static RUN_PROOF:Mutex<u8> = Mutex::new(0);
		
		let handler_source:Arc<dyn Fn() -> Result<(), Box<dyn Error>> + Send + Sync + 'static> = Arc::new(
			|| {
				*RUN_PROOF.lock().unwrap() += 1;
				Ok(())
			}
		);
		let mut handler:TaskHandler = handler_source.into_handler();
		let mut event:Event = Event::default();
		
		for index in 1..=64 {
			handler.run(&mut event).unwrap();
			assert_eq!(*RUN_PROOF.lock().unwrap(), index);
			assert_eq!(event.expired, false);
		}
	}
	
	#[test]
	fn handler_source_arc_fn_1args_no_result_specified_source() {
		static RUN_PROOF:Mutex<u8> = Mutex::new(0);
		
		let handler_source:Arc<dyn Fn(&mut Event)  + Send + Sync + 'static> = Arc::new(
			|event| {
				*RUN_PROOF.lock().unwrap() += 1;
				if *RUN_PROOF.lock().unwrap() == 50 {{ event.expire(); }}
			}
		);
		let mut handler:TaskHandler = handler_source.into_handler();
		let mut event:Event = Event::default();
		
		for index in 1..=64 {
			handler.run(&mut event).unwrap();
			assert_eq!(*RUN_PROOF.lock().unwrap(), index);
			assert_eq!(event.expired, index >= 50);
		}
	}
	
	#[test]
	fn handler_source_arc_fn_1args_has_result_specified_source() {
		static RUN_PROOF:Mutex<u8> = Mutex::new(0);
		
		let handler_source:Arc<dyn Fn(&mut Event) -> Result<(), Box<dyn Error>> + Send + Sync + 'static> = Arc::new(
			|event| {
				*RUN_PROOF.lock().unwrap() += 1;
				if *RUN_PROOF.lock().unwrap() == 50 {{ event.expire(); }}
				Ok(())
			}
		);
		let mut handler:TaskHandler = handler_source.into_handler();
		let mut event:Event = Event::default();
		
		for index in 1..=64 {
			handler.run(&mut event).unwrap();
			assert_eq!(*RUN_PROOF.lock().unwrap(), index);
			assert_eq!(event.expired, index >= 50);
		}
	}
	
}