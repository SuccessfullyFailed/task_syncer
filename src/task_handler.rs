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
	pub fn new<Source:TaskHandlerSource>(source:Source) -> TaskHandler {
		source.into_handler()
	}

	pub fn run(&mut self, event:&mut Event) -> Result<(), Box<dyn Error>> {
		match self {
			TaskHandler::None => Ok(()),
			TaskHandler::Fn(handler) => handler(event),
			TaskHandler::FnMut(handler_mut) => handler_mut(event),
			TaskHandler::Task(task) => {
				let result:Result<(), Box<dyn Error>> = task.run();

				result
			},
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
			TaskHandler::List((handlers, handler_index)) => {
				let result:Result<(), Box<dyn Error>> = handlers.get_mut(*handler_index).map(|handler| handler.run(event)).unwrap_or(Ok(()));
				if event.expired {
					*handler_index += 1;
					*event = Event::default();
					if *handler_index >= handlers.len() {
						event.expired = true;
					}
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