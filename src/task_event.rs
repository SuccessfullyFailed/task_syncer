use std::{ error::Error, time::{ Duration, Instant } };



pub struct Event {
	pub(crate) target_instant:Instant,
	pub(crate) repeat:bool
}
impl Event {

	/* CONSTRUCTOR METHODS */

	/// Create a new, empty default event.
	pub fn new() -> Event {
		Event {
			target_instant: Instant::now(),
			repeat: true
		}
	}



	/* USAGE METHODS */

	/// Check if the task is scheduled to run.
	pub(crate) fn should_run(&self, now:&Instant) -> bool {
		self.target_instant < *now
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
}