use std::{ error::Error, time::{ Duration, Instant } };



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
			self.pause_time = Some(now.clone());
		}
	}

	/// Resume the event. Adds the paused time to the trigger timer.
	pub fn resume(&mut self) {
		if let Some(pause_time) = self.pause_time {
			self.target_instant += pause_time.elapsed();
			self.pause_time = None;
		}
	}
}