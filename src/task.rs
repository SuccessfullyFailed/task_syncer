use std::{ error::Error, time::{ Duration, Instant } };
use crate::TaskScheduler;



pub struct TaskHandler(Box<dyn FnMut(&TaskScheduler, &mut TaskEvent) -> Result<(), Box<dyn Error>> + Send + Sync + 'static>);
impl<T:FnMut(&TaskScheduler, &mut TaskEvent) -> Result<(), Box<dyn Error>> + Send + Sync + 'static> From<T> for TaskHandler {
	fn from(value:T) -> Self {
		TaskHandler(Box::new(value))
	}
}



pub struct Task {
	pub(crate) name:String,
	pub(crate) event:TaskEvent,
	handler:TaskHandler,
	catch_handler:Option<Box<dyn Fn(&Box<dyn Error>) + Send + Sync + 'static>>,
	finally_handler:Option<Box<dyn Fn(&Result<(), Box<dyn Error>>) + Send + Sync + 'static>>
}
impl Task {

	/* CONSTRUCTOR METHODS */

	/// Create a new task.
	pub fn new<Handler:Into<TaskHandler>>(name:&str, handler:Handler) -> Task {
		Task {
			name: name.to_string(),
			event: TaskEvent::default(),
			handler: handler.into(),
			catch_handler: None,
			finally_handler: None
		}
	}



	/* USAGE METHODS */

	/// Run the task.
	pub fn run(&mut self, now:&Instant, scheduler:&TaskScheduler) -> Result<(), Box<dyn Error>> {

		// If the task should not yet, return.
		if self.event.expired || &self.event.trigger_target > now {
			return Ok(());
		}

		// Run handler.
		self.event.repeat = false;
		let result:Result<(), Box<dyn Error>> = (self.handler.0)(scheduler, &mut self.event);
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

	/// Let the task handle any changes after the system has been paused for the given duration.
	pub fn handle_paused_duration(&mut self, duration:&Duration) {
		self.event.delay(*duration);
	}
}



pub struct TaskEvent {
	pub(crate) expired:bool,
	pub(crate) repeat:bool,
	pub(crate) trigger_target:Instant,
	pub(crate) require_accurate_timing:bool
}
impl TaskEvent {

	/// Set the event as expired.
	pub fn expire(&mut self) {
		self.expired = true;
	}

	/// Delay the target trigger time by the given duration.
	pub fn delay(&mut self, delay:Duration) {
		self.trigger_target += delay;
		self.require_accurate_timing = false;
	}

	/// Delay the target trigger time by the given duration.
	/// Uses a more accurate and cpu-consuming way to match the timing.
	pub fn delay_accurate(&mut self, delay:Duration) {
		self.trigger_target += delay;
		self.require_accurate_timing = true;
	}

	/// Set the event to run again.
	pub fn repeat(&mut self) {
		self.repeat = true;
	}

	/// Set the event to run again.
	/// Always returns 'Ok(())' so it can be used at the end of a handler.
	pub fn repeat_r(&mut self) -> Result<(), Box<dyn Error>> {
		self.repeat = true;
		Ok(())
	}

	/// Reschedule the event to run again.
	/// Combines the 'delay' and 'run_again' function.
	pub fn reschedule(&mut self, delay:Duration) {
		self.delay(delay);
		self.repeat();
	}

	/// Reschedule the event to run again.
	/// Combines the 'delay' and 'run_again' function.
	/// Uses a more accurate and cpu-consuming way to match the timing.
	pub fn reschedule_accurate(&mut self, delay:Duration) {
		self.delay_accurate(delay);
		self.repeat();
	}

	/// Reschedule the event to run again.
	/// Combines the 'delay' and 'run_again' function. Always returns 'Ok(())' so it can be used at the end of a handler.
	pub fn reschedule_r(&mut self, delay:Duration) -> Result<(), Box<dyn Error>> {
		self.reschedule(delay);
		Ok(())
	}

	/// Reschedule the event to run again.
	/// Combines the 'delay' and 'run_again' function. Always returns 'Ok(())' so it can be used at the end of a handler.
	/// Uses a more accurate and cpu-consuming way to match the timing.
	pub fn reschedule_accurate_r(&mut self, delay:Duration) -> Result<(), Box<dyn Error>> {
		self.reschedule_accurate(delay);
		Ok(())
	}
}
impl Default for TaskEvent {
	fn default() -> Self {
		TaskEvent {
			expired: false,
			repeat: true,
			trigger_target: Instant::now(),
			require_accurate_timing: false
		}
	}
}