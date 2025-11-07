use crate::{ DEFAULT_DUPLICATE_HANDLER, DuplicateHandler, TaskLike, TaskScheduler, TaskType };
use std::{ error::Error, time::Instant };



type HandlerResult = Result<(), Box<dyn Error>>;
type Handler = Box<dyn Fn(&TaskScheduler) -> HandlerResult + Send + Sync >;
type ErrorHandler = Box<dyn Fn(&TaskScheduler, Box<dyn Error>) + Send + Sync >;



pub struct EventSubscription {
	name:String,
	event_name:String,
	duplicate_handler:DuplicateHandler,

	handler:Handler,
	catch_handler:ErrorHandler
}
impl EventSubscription {

	/// Create a new task.
	pub fn new<T:Fn(&TaskScheduler) -> HandlerResult + Send + Sync  + 'static>(name:&str, event_name:&str, handler:T) -> EventSubscription {
		EventSubscription {
			name: name.to_string(),
			event_name: event_name.to_string(),
			duplicate_handler: DEFAULT_DUPLICATE_HANDLER,

			handler: Box::new(handler),
			catch_handler: Box::new(|_, error| eprintln!("{error}"))
		}
	}

	/// Return self with a duplicate handler.
	pub fn with_duplicate_handler(mut self, duplicate_handler:DuplicateHandler) -> Self {
		self.duplicate_handler = duplicate_handler;
		self
	}

	/// Return self with a new error handler.
	pub fn catch<T:Fn(&TaskScheduler, Box<dyn Error>) + Send + Sync  + 'static>(mut self, handler:T) -> Self {
		self.catch_handler = Box::new(handler);
		self
	}
}
impl TaskLike for EventSubscription {
	
	/* PROPERTY GETTER METHODS */

	/// The name of the task.
	fn name(&self) -> &str {
		&self.name
	}

	/// The type-name of the task.
	fn task_type(&self) -> TaskType {
		TaskType::Subscription
	}

	/// Get the duplicate handler of the task.
	fn duplicate_handler(&self) -> &DuplicateHandler {
		&self.duplicate_handler
	}

	/// Wether or not the task is expired.
	fn expired(&self) -> bool {
		false
	}

	/// Check if the task is scheduled to run.
	fn should_run(&self, _now:&Instant, triggered_events:&[String]) -> bool {
		triggered_events.contains(&self.event_name)
	}


	
	/* USAGE METHODS */

	/// Run the task.
	fn run(&mut self, task_scheduler:&TaskScheduler) {
		let result:HandlerResult = (self.handler)(task_scheduler);
		if let Err(error) = result {
			(self.catch_handler)(task_scheduler, error);
		}
	}

	/// Pause the task, temporarily disabling it.
	fn pause(&mut self, _now:&Instant) {}

	/// Resume the task, reinstating functionality to it.
	fn resume(&mut self, _now:&Instant) {}
}