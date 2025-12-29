use crate::{ TaskHandlerSource, task_handler::TaskHandler };
use std::{ error::Error, time::{ Duration, Instant } };



pub struct Task {
	pub(crate) name:String,
	pub(crate) event:TaskEvent,
	handler:Box<TaskHandler>,
	catch_handler:Option<Box<dyn Fn(&Box<dyn Error>) + Send + Sync + 'static>>,
	finally_handler:Option<Box<dyn Fn(&Result<(), Box<dyn Error>>) + Send + Sync + 'static>>
}
impl Task {

	/* CONSTRUCTOR METHODS */

	/// Create a new task.
	pub fn new<T:TaskHandlerSource + 'static>(name:&str, handler:T) -> Task {
		Task {
			name: name.to_string(),
			event: TaskEvent::default(),
			handler: Box::new(handler.into_handler()),
			catch_handler: None,
			finally_handler: None
		}
	}



	/* USAGE METHODS */

	/// Run the task.
	pub fn run(&mut self, now:&Instant) -> Result<(), Box<dyn Error>> {

		// If the task should not yet, return.
		if self.event.expired || &self.event.trigger_target > now {
			return Ok(());
		}

		// Run handler.
		self.event.repeat = false;
		let result:Result<(), Box<dyn Error>> = self.handler.run(now, &mut self.event);
		if !self.event.repeat {
			self.event.expire();
		}

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



pub struct TaskEvent {
	pub(crate) expired:bool,
	pub(crate) repeat:bool,
	pub(crate) trigger_target:Instant
}
impl TaskEvent {

	/// Set the event as expired.
	pub fn expire(&mut self) {
		self.expired = true;
	}

	/// Delay the target trigger time by the given duration.
	pub fn delay(&mut self, delay:Duration) {
		self.trigger_target += delay;
	}

	/// Set the event to run again.
	pub fn repeat(&mut self) {
		self.repeat = true;
	}

	/// Set the event to run again. Always returns 'Ok(())' so it can be used at the end of a handler.
	pub fn repeated(&mut self) -> Result<(), Box<dyn Error>> {
		self.repeat = true;
		Ok(())
	}

	/// Reschedule the event to run again. Combines the 'delay' and 'run_again' function.
	pub fn reschedule(&mut self, delay:Duration) {
		self.delay(delay);
		self.repeat();
	}

	/// Reschedule the event to run again. Combines the 'delay' and 'run_again' function. Always returns 'Ok(())' so it can be used at the end of a handler.
	pub fn rescheduled(&mut self, delay:Duration) -> Result<(), Box<dyn Error>> {
		self.delay(delay);
		self.repeat();
		Ok(())
	}
}
impl Default for TaskEvent {
	fn default() -> Self {
		TaskEvent {
			expired: false,
			repeat: true,
			trigger_target: Instant::now()
		}
	}
}