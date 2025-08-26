use std::{ error::Error, time::{ Duration, Instant } };
use crate::Event;



pub type HandlerResult = Result<(), Box<dyn Error>>;
pub type Handler = Box<dyn Fn(&mut Event) -> HandlerResult>;
pub type ErrorHandler = Box<dyn Fn(&mut Event, Box<dyn Error>)>;



pub struct Task {
	name:String,
	event:Event,
	expired:bool,
	handler:Handler,
	then_handlers:Vec<Handler>,
	error_handler:ErrorHandler
}
impl Task {

	/* CONSTRUCTOR METHODS */

	/// Create a new task.
	pub fn new<T>(name:&str, handler:T) -> Task where T:Fn(&mut Event) -> HandlerResult + 'static {
		Task {
			name: name.to_string(),
			event: Event::new(),
			expired: false,
			handler: Box::new(handler),
			then_handlers: Vec::new(),
			error_handler: Box::new(|_, error| eprintln!("{error}")) // Ensures that all other tasks continue as scheduled.
		}
	}

	/// Return self with a new error handler.
	pub fn on_error<T>(mut self, handler:T) -> Self where T:Fn(&mut Event, Box<dyn Error>) + 'static {
		self.error_handler = Box::new(handler);
		self
	}

	/// Return self with an afterwards handler.
	pub fn then<T>(mut self, handler:T) -> Self where T:Fn(&mut Event) -> HandlerResult + 'static {
		self.then_handlers.push(Box::new(handler));
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
		let result:HandlerResult = (self.handler)(&mut self.event);
		if let Err(error) = result {
			(self.error_handler)(&mut self.event, error);
		}

		// If task should not repeat, switch to next handler or set as expired.
		if !self.event.repeat {
			if !self.then_handlers.is_empty() {
				self.handler = self.then_handlers.remove(0);
				self.event = Event::new();
			} else {
				self.expired = true;
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