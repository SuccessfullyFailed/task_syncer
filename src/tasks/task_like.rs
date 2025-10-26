use crate::TaskScheduler;
use std::time::Instant;



pub enum DuplicateHandler { KeepAll, KeepOld, KeepNew }
pub const DEFAULT_DUPLICATE_HANDLER:DuplicateHandler = DuplicateHandler::KeepAll;



pub trait TaskLike {
	
	/* PROPERTY GETTER METHODS */

	/// The name of the task.
	fn name(&self) -> &str;

	/// Get the duplicate handler of the task.
	fn duplicate_handler(&self) -> &DuplicateHandler;

	/// Wether or not the task is expired.
	fn expired(&self) -> bool;

	/// Check if the task is scheduled to run.
	fn should_run(&self, now:&Instant, triggered_events:&[String]) -> bool;



	/* USAGE METHODS */

	/// Run the event.
	fn run(&mut self, task_scheduler:&TaskScheduler);

	/// Pause the task, temporarily disabling it.
	fn pause(&mut self, now:&Instant);

	/// Resume the task, reinstating functionality to it.
	fn resume(&mut self, now:&Instant);
}