use crate::{ BoxedTaskHandlerSource, task_handler::TaskHandler };
use std::error::Error;



pub struct Task {
	pub(crate) name:String,
	pub(crate) event:Event,
	handler:Box<TaskHandler>,
	catch_handler:Option<Box<dyn Fn(&Box<dyn Error>) + Send + Sync + 'static>>,
	finally_handler:Option<Box<dyn Fn(&Result<(), Box<dyn Error>>) + Send + Sync + 'static>>
}
impl Task {

	/* CONSTRUCTOR METHODS */

	/// Create a new task.
	pub fn new<T:'static>(name:&str, handler:T) -> Task where Box<T>:BoxedTaskHandlerSource {
		Task {
			name: name.to_string(),
			event: Event::default(),
			handler: Box::new(vec![Box::new(handler)].into_handler()),
			catch_handler: None,
			finally_handler: None
		}
	}



	/* USAGE METHODS */

	/// Run the task.
	pub fn run(&mut self) -> Result<(), Box<dyn Error>> {

		// Run handler.
		let result:Result<(), Box<dyn Error>> = self.handler.run(&mut self.event);

		// Custom error handler.
		if let Err(error) = &result {
			if let Some(catch_handler) = &self.catch_handler {
				catch_handler(error);
			}
		}

		// Expiration handler.
		if self.event.expired {
			if let Some(finally_handler) = &self.finally_handler {
				finally_handler(&result);
			}
		}

		// Return result.
		result
	}
}



pub struct Event {
	pub(crate) expired:bool
}
impl Event {

	/* SETTER METHODS */

	/// Set the event as expired.
	pub fn expire(&mut self) {
		self.expired = true;
	}
}
impl Default for Event {
	fn default() -> Self {
		Event {
			expired: false
		}
	}
}