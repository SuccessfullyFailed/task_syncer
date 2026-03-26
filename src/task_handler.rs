use std::{ error::Error, ops::Range, time::Instant };
use crate::{ TaskEvent, Task };



pub enum TaskHandler {
	None,
	Fn(Box<dyn FnMut(&mut TaskEvent) -> Result<(), Box<dyn Error>> + Send + Sync + 'static>),
	Task(Task),
	Repeat((Box<TaskHandler>, Range<usize>)),
	List((Vec<TaskHandler>, usize))
}
impl TaskHandler {

	/* CONSTRUCTOR METHODS */

	/// Create a new task-handler.
	pub fn new<Source:TaskHandlerSource>(source:Source) -> TaskHandler {
		source.into_handler()
	}



	/* USAGE METHODS */
	
	/// Run the task handler. Handles event updating and expiration.
	pub fn run(&mut self, now:&Instant, event:&mut TaskEvent) -> Result<(), Box<dyn Error>> {
		match self {

			// No handler, return success.
			TaskHandler::None => {
				event.expired = true;
				Ok(())
			},

			// Function handler, return function result.
			TaskHandler::Fn(handler) => handler(event),

			// task handler, run task until task event expires.
			TaskHandler::Task(task) => {
				let result:Result<(), Box<dyn Error>> = task.run(now);
				if task.event.expired {
					event.expired = true;
				}
				result
			},

			// Repeat handler the given amount.
			TaskHandler::Repeat((handler, range)) => {
				if range.start < range.end {
					let result:Result<(), Box<dyn Error>> = handler.run(now, event);
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
				let result:Result<(), Box<dyn Error>> = handlers.get_mut(*handler_index).map(|handler| handler.run(now, event)).unwrap_or(Ok(()));
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



pub trait TaskHandlerSource:Sized + Send + Sync + 'static {
	#[allow(unused_mut)]
	fn into_handler(mut self) -> TaskHandler {
		TaskHandler::None
	}
}
impl<T:TaskHandlerSource + Clone + 'static, const SIZE:usize> TaskHandlerSource for [T; SIZE] {
	fn into_handler(self) -> TaskHandler {
		self.to_vec().into_handler()
	}
}
impl<T:TaskHandlerSource + 'static> TaskHandlerSource for Vec<T> {
	fn into_handler(self) -> TaskHandler {
		TaskHandler::List((self.into_iter().map(|source| source.into_handler()).collect(), 0))
	}
}
impl<T:FnMut(&mut TaskEvent) -> Result<(), Box<dyn Error>> + Send + Sync + 'static> TaskHandlerSource for T {
	fn into_handler(self) -> TaskHandler {
		TaskHandler::Fn(Box::new(self))
	}
}