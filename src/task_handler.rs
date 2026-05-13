use std::{ error::Error, ops::Range, time::Instant };
use crate::{ Task, TaskEvent, TaskScheduler };



pub enum TaskHandler {
	None,
	Fn(Box<dyn FnMut(&TaskScheduler, &mut TaskEvent) -> Result<(), Box<dyn Error>> + Send + Sync + 'static>),
	Task(Task),
	Repeat((Box<TaskHandler>, Range<usize>)),
	List((Vec<TaskHandler>, usize))
}
impl TaskHandler {

	/* CONSTRUCTOR METHODS */

	/// Create a new task-handler.
	pub fn new<Handler>(source:Handler) -> TaskHandler where TaskHandler:From<Handler> {
		TaskHandler::from(source)
	}



	/* USAGE METHODS */
	
	/// Run the task handler. Handles event updating and expiration.
	pub fn run(&mut self, now:&Instant, scheduler:&TaskScheduler, event:&mut TaskEvent) -> Result<(), Box<dyn Error>> {
		match self {

			// No handler, return success.
			TaskHandler::None => {
				event.expired = true;
				Ok(())
			},

			// Function handler, return function result.
			TaskHandler::Fn(handler) => handler(scheduler, event),

			// Task handler, run task until task event expires.
			TaskHandler::Task(task) => {
				let result:Result<(), Box<dyn Error>> = task.run(now, scheduler);
				if task.event.expired {
					event.expired = true;
				}
				result
			},

			// Repeat handler the given amount.
			TaskHandler::Repeat((handler, range)) => {
				if range.start < range.end {
					let result:Result<(), Box<dyn Error>> = handler.run(now, scheduler, event);
					range.start += 1;
					if range.start >= range.end {
						event.expired = true;
					}
					result
				} else {
					event.expired = true;
					Ok(())
				}
			},

			// Run through a list of handlers, passing on to the next once the first expires.
			TaskHandler::List((handlers, handler_index)) => {
				let result:Result<(), Box<dyn Error>> = handlers.get_mut(*handler_index).map(|handler| handler.run(now, scheduler, event)).unwrap_or(Ok(()));
				if event.expired {
					*handler_index += 1;
					*event = TaskEvent::default();
				}
				if *handler_index >= handlers.len() {
					event.expired = true;
				}
				result
			}
		}
	}
}
impl<T:FnMut(&TaskScheduler, &mut TaskEvent) -> Result<(), Box<dyn Error>> + Send + Sync + 'static> From<T> for TaskHandler {
	fn from(value:T) -> Self {
		TaskHandler::Fn(
			Box::new(value)
		)
	}
}
impl<T:FnMut(&TaskScheduler, &mut TaskEvent) -> Result<(), Box<dyn Error>> + Send + Sync + 'static> From<(T, usize)> for TaskHandler {
	fn from(value:(T, usize)) -> Self {
		TaskHandler::Repeat((
			Box::new(TaskHandler::from(value.0)),
			0..value.1
		))
	}
}
impl<T> From<Vec<T>> for TaskHandler where TaskHandler:From<T> {
	fn from(value:Vec<T>) -> Self {
		TaskHandler::List((
			value.into_iter().map(TaskHandler::from).collect(),
			0
		))
	}
}
impl<T, const SIZE:usize> From<[T; SIZE]> for TaskHandler where TaskHandler:From<T> {
	fn from(value:[T; SIZE]) -> Self {
		TaskHandler::List((
			value.into_iter().map(TaskHandler::from).collect(),
			0
		))
	}
}