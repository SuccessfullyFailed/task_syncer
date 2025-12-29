use crate::{ Event, Task };
use std::error::Error;



pub enum TaskHandler {
	None,
	Fn(Box<dyn Fn(&mut Event) -> Result<(), Box<dyn Error>> + Send + Sync + 'static>),
	FnMut(Box<dyn FnMut(&mut Event) -> Result<(), Box<dyn Error>> + Send + Sync + 'static>),
	Task(Task),
	Repeat((Box<TaskHandler>, usize, usize)),
	List((Vec<TaskHandler>, usize))
}
impl TaskHandler {

	/* CONSTRUCTOR METHODS */

	/// Create a new task-handler.
	pub fn new<Source:BoxedTaskHandlerSource>(source:Source) -> TaskHandler {
		source.into_handler()
	}



	/* USAGE METHODS */
	
	/// Run the task handler. Handles event updating and expiration.
	pub fn run(&mut self, event:&mut Event) -> Result<(), Box<dyn Error>> {
		match self {

			// No handler, return success.
			TaskHandler::None => Ok(()),

			// Function handlers, return function result.
			TaskHandler::Fn(handler) => handler(event),
			TaskHandler::FnMut(handler_mut) => handler_mut(event),

			// task handler, run task until task event expires.
			TaskHandler::Task(task) => {
				let result:Result<(), Box<dyn Error>> = task.run();
				if task.event.expired {
					event.expired = true;
				}
				result
			},

			// Repeat handler the given amount.
			TaskHandler::Repeat((handler, index, repeat_count)) => {
				if *index < *repeat_count {
					let result:Result<(), Box<dyn Error>> = handler.run(event);
					*index += 1;
					if *index >= *repeat_count {
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
				let result:Result<(), Box<dyn Error>> = handlers.get_mut(*handler_index).map(|handler| handler.run(event)).unwrap_or(Ok(()));
				if event.expired {
					*handler_index += 1;
					*event = Event::default();
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
	fn into_handler(self) -> TaskHandler {
		TaskHandler::None	
	}
}
impl<T:Send + Sync + 'static> TaskHandlerSource for T where Box<T>:BoxedTaskHandlerSource {
	fn into_handler(self) -> TaskHandler {
		Box::new(self).into_handler()
	}
}



pub trait BoxedTaskHandlerSource:Sized + Send + Sync + 'static {
	#[allow(unused_mut)]
	fn into_handler(mut self) -> TaskHandler {
		TaskHandler::None
	}
}
impl<T:BoxedTaskHandlerSource + Clone + 'static, const SIZE:usize> BoxedTaskHandlerSource for [T; SIZE] {
	fn into_handler(self) -> TaskHandler {
		self.to_vec().into_handler()
	}
}
impl<T:BoxedTaskHandlerSource + 'static> BoxedTaskHandlerSource for Vec<T> {
	fn into_handler(self) -> TaskHandler {
		TaskHandler::List((self.into_iter().map(|source| source.into_handler()).collect(), 0))
	}
}