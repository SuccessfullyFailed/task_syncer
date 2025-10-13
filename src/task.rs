use std::{ error::Error, time::{ Duration, Instant } };
use crate::Event;



pub type HandlerResult = Result<(), Box<dyn Error>>;
pub type Handler = Box<dyn Fn(&mut Event) -> HandlerResult + Send>;
pub type ErrorHandler = Box<dyn Fn(&mut Event, Box<dyn Error>) + Send>;



pub struct Task {
	name:String,
	event:Event,
	expired:bool,
	handler_index:usize,
	handlers:Vec<Handler>,
	catch_handler:ErrorHandler,
	finally:Vec<Handler>
}
impl Task {

	/* CONSTRUCTOR METHODS */

	/// Create a new task.
	pub fn new<T>(name:&str, handler:T) -> Task where T:Fn(&mut Event) -> HandlerResult + Send + 'static {
		Task {
			name: name.to_string(),
			event: Event::new(),
			expired: false,
			handler_index: 0,
			handlers: vec![Box::new(handler)],
			catch_handler: Box::new(|_, error| eprintln!("{error}")), // Ensures that all other tasks continue as scheduled.
			finally: Vec::new()
		}
	}

	/// Return self with a new handler that executes after the previous one has expired.
	pub fn then<T>(mut self, handler:T) -> Self where T:Fn(&mut Event) -> HandlerResult + Send + 'static {
		self.handlers.push(Box::new(handler));
		self
	}

	/// Return self with a new error handler.
	pub fn catch<T>(mut self, handler:T) -> Self where T:Fn(&mut Event, Box<dyn Error>) + Send + 'static {
		self.catch_handler = Box::new(handler);
		self
	}

	/// Return self with a new handler that executes once the entire task has finished or expired.
	pub fn finally<T>(mut self, handler:T) -> Self where T:Fn(&mut Event) -> HandlerResult + Send + 'static {
		self.finally.push(Box::new(handler));
		self
	}



	/* PROPERTY GETTER METHODS */

	/// The name of the task.
	pub fn name(&self) -> &str {
		&self.name
	}

	/// Wether or not the task is expired.
	pub(crate) fn expired(&self) -> bool {
		self.expired
	}

	/// Check if the task is scheduled to run.
	pub(crate) fn should_run(&self, now:&Instant) -> bool {
		!self.expired && self.event.should_run(now)
	}



	/* USAGE METHODS */

	/// Run the task.
	pub(crate) fn run(&mut self) {

		// Run handlers.
		self.event.repeat = false;
		let result:HandlerResult = (self.handlers[self.handler_index])(&mut self.event);
		if let Err(error) = result {
			(self.catch_handler)(&mut self.event, error);
			self.expired = true;
		}

		// If task should not repeat, switch to next handler or set as expired.
		if !self.event.repeat {
			self.event = Event::new();
			self.handler_index += 1;
			if self.handler_index >= self.handlers.len() {
				self.expired = true;
			}
		}

		// If expired, run finally handlers.
		for handler in &self.finally {
			let result:HandlerResult = handler(&mut self.event);
			if let Err(error) = result {
				(self.catch_handler)(&mut self.event, error);
			}
		}
	}

	/// Delay the task.
	pub fn delay(mut self, delay:Duration) -> Self {
		self.event.delay(delay);
		self
	}

	/// Pause the event. Stores the current time and adds the paused time to the trigger timer upon resume.
	pub fn pause(&mut self, now:&Instant) {
		self.event.pause(now);
	}

	/// Resume the event. Adds the paused time to the trigger timer.
	pub fn resume(&mut self) {
		self.event.resume();
	}
}