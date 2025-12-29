use crate::task_handler::TaskHandler;
use std::error::Error;



pub struct Task {
	pub(crate) name:String,
	pub(crate) event:Event,
	handler_index:usize,
	handlers:Vec<TaskHandler>,
	catch_handler:Option<Box<dyn Fn(&Box<dyn Error>) + Send + Sync + 'static>>,
	finally_handler:Option<Box<dyn Fn(&Result<(), Box<dyn Error>>) + Send + Sync + 'static>>
}
impl Task {

	/// Run the task.
	pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
		match self.handlers.get_mut(self.handler_index) {
			Some(handler) => {
				let result:Result<(), Box<dyn Error>> = handler.run(&mut self.event);
				if let Err(error) = &result {
					if let Some(catch_handler) = &self.catch_handler {
						catch_handler(error);
					}
				}
				if self.event.expired {
					self.handler_index += 1;
					if self.handler_index < self.handlers.len() {
						self.event = Event::default();
					} else if let Some(finally_handler) = &self.finally_handler {
						finally_handler(&result);
					}
				}
				result
			},
			None => {
				self.event.expired = true;
				Ok(())
			}
		}
	}
}



pub struct Event {
	pub(crate) expired:bool
}
impl Default for Event {
	fn default() -> Self {
		Event {
			expired: false
		}
	}
}