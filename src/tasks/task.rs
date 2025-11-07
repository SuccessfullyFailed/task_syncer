use crate::{ DEFAULT_DUPLICATE_HANDLER, DuplicateHandler, TaskLike, TaskScheduler, TaskType };
use std::{ error::Error, time::{ Duration, Instant } };



type HandlerResult = Result<(), Box<dyn Error>>;
type Handler = Box<dyn Fn(&TaskScheduler, &mut Event) -> HandlerResult + Send + Sync >;
type ErrorHandler = Box<dyn Fn(&TaskScheduler, &mut Event, Box<dyn Error>) + Send + Sync >;



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
	pub fn new<T:Fn(&TaskScheduler, &mut Event) -> HandlerResult + Send + Sync + 'static>(name:&str, handler:T) -> Task {
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
	pub fn then<T:Fn(&TaskScheduler, &mut Event) -> HandlerResult + Send + Sync  + 'static>(mut self, handler:T) -> Self {
		self.handlers.push(Box::new(handler));
		self
	}

	/// Return self with a new error handler.
	pub fn catch<T:Fn(&TaskScheduler, &mut Event, Box<dyn Error>) + Send + Sync  + 'static>(mut self, handler:T) -> Self {
		self.catch_handler = Box::new(handler);
		self
	}

	/// Return self with a new handler that executes once the entire task has finished or expired.
	pub fn finally<T:Fn(&TaskScheduler, &mut Event) -> HandlerResult + Send + Sync  + 'static>(mut self, handler:T) -> Self {
		self.finally.push(Box::new(handler));
		self
	}



	/* EVENT USAGE METHODS */

	/// Delay the task.
	pub fn delay(mut self, delay:Duration) -> Self {
		self.event.delay(delay);
		self
	}
}
impl TaskLike for Task {
	
	/* PROPERTY GETTER METHODS */
	
	/// The name of the task.
	fn name(&self) -> &str {
		&self.name
	}

	/// The type-name of the task.
	fn task_type(&self) -> TaskType {
		TaskType::Task
	}

	/// Get the duplicate handler of the task.
	fn duplicate_handler(&self) -> &DuplicateHandler {
		&self.duplicate_handler
	}

	/// Wether or not the task is expired.
	fn expired(&self) -> bool {
		self.expired
	}

	/// Check if the task is scheduled to run.
	fn should_run(&self, now:&Instant, _triggered_events:&[String]) -> bool {
		!self.expired && self.event.should_run(now)
	}


	
	/* USAGE METHODS */

	/// Run the task.
	fn run(&mut self, task_scheduler:&TaskScheduler) {

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

	/// Pause the task, temporarily disabling it.
	fn pause(&mut self, now:&Instant) {
		self.event.pause(now);
	}

	/// Resume the task, reinstating functionality to it.
	fn resume(&mut self, now:&Instant) {
		self.event.resume(now);
	}
}



pub struct Event {
	pub(crate) target_instant:Instant,
	pub(crate) repeat:bool,
	pub(crate) pause_time:Option<Instant>
}
impl Event {

	/* CONSTRUCTOR METHODS */

	/// Create a new, empty default event.
	pub fn new() -> Event {
		Event {
			target_instant: Instant::now(),
			repeat: true,
			pause_time: None
		}
	}



	/* USAGE METHODS */

	/// Check if the task is scheduled to run.
	pub(crate) fn should_run(&self, now:&Instant) -> bool {
		self.pause_time.is_none() && self.target_instant < *now
	}

	/// Repeat the event.
	pub fn repeat(&mut self) {
		self.repeat = true;
	}

	/// Delay the event.
	pub fn delay(&mut self, duration:Duration) {
		self.target_instant += duration;
	}

	/// Reschedule the event. Returns Ok(()) so it can easily be used at the end of handlers.
	pub fn reschedule(&mut self, delay:Duration) -> Result<(), Box<dyn Error>> {
		self.delay(delay);
		self.repeat();
		Ok(())
	}

	/// Pause the event. Stores the current time and adds the paused time to the trigger timer upon resume.
	pub fn pause(&mut self, now:&Instant) {
		if self.pause_time.is_none() {
			self.pause_time = Some(*now);
		}
	}

	/// Resume the event. Adds the paused time to the trigger timer.
	pub fn resume(&mut self, now:&Instant) {
		if let Some(pause_time) = self.pause_time {
			self.target_instant += now.duration_since(pause_time);
			self.pause_time = None;
		}
	}
}
impl Default for Event {
	fn default() -> Self {
		Event::new()
	}
}