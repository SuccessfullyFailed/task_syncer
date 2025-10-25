use std::{ error::Error, time::{ Duration, Instant } };
use crate::{Event, TaskScheduler};



pub type HandlerResult = Result<(), Box<dyn Error>>;
pub type Handler = Box<dyn Fn(&TaskScheduler, &mut Event) -> HandlerResult + Send>;
pub type ErrorHandler = Box<dyn Fn(&TaskScheduler, &mut Event, Box<dyn Error>) + Send>;
pub enum DuplicateHandler { KeepAll, KeepOld, KeepNew }
const DEFAULT_DUPLICATE_HANDLER:DuplicateHandler = DuplicateHandler::KeepAll;



pub struct Task {
	name:String,
	duplicate_handler:DuplicateHandler,

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
	pub fn new<T>(name:&str, handler:T) -> Task where T:Fn(&TaskScheduler, &mut Event) -> HandlerResult + Send + 'static {
		Task {
			name: name.to_string(),
			duplicate_handler: DEFAULT_DUPLICATE_HANDLER,

			event: Event::new(),
			expired: false,

			handler_index: 0,
			handlers: vec![Box::new(handler)],
			catch_handler: Box::new(|_, _, error| eprintln!("{error}")), // Ensures that all other tasks continue as scheduled.
			finally: Vec::new()
		}
	}

	/// Return self with a duplicate handler.
	pub fn with_duplicate_handler(mut self, duplicate_handler:DuplicateHandler) -> Self {
		self.duplicate_handler = duplicate_handler;
		self
	}

	/// Return self with a new handler that executes after the previous one has expired.
	pub fn then<T>(mut self, handler:T) -> Self where T:Fn(&TaskScheduler, &mut Event) -> HandlerResult + Send + 'static {
		self.handlers.push(Box::new(handler));
		self
	}

	/// Return self with a new error handler.
	pub fn catch<T>(mut self, handler:T) -> Self where T:Fn(&TaskScheduler, &mut Event, Box<dyn Error>) + Send + 'static {
		self.catch_handler = Box::new(handler);
		self
	}

	/// Return self with a new handler that executes once the entire task has finished or expired.
	pub fn finally<T>(mut self, handler:T) -> Self where T:Fn(&TaskScheduler, &mut Event) -> HandlerResult + Send + 'static {
		self.finally.push(Box::new(handler));
		self
	}



	/* PROPERTY GETTER METHODS */

	/// The name of the task.
	pub fn name(&self) -> &str {
		&self.name
	}

	/// Get the duplicate handler of the task.
	pub fn duplicate_handler(&self) -> &DuplicateHandler {
		&self.duplicate_handler
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
	pub(crate) fn run(&mut self, task_scheduler:&TaskScheduler) {

		// Run handlers.
		self.event.repeat = false;
		let result:HandlerResult = (self.handlers[self.handler_index])(task_scheduler, &mut self.event);
		if let Err(error) = result {
			(self.catch_handler)(task_scheduler, &mut self.event, error);
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
		if self.expired {
			for handler in &self.finally {
				let result:HandlerResult = handler(task_scheduler, &mut self.event);
				if let Err(error) = result {
					(self.catch_handler)(task_scheduler, &mut self.event, error);
				}
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