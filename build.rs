use file_ref::FileRef;



const GENERATED_HANDLER_IMPLEMENTATIONS_FILE:FileRef = FileRef::new_const("src/task_handler_generated_implementations.rs");
const GENERATED_HANDLER_TESTS_FILE:FileRef = FileRef::new_const("src/task_handler_generated_implementations_u.rs");
const GENERATED_HANDLER_MIN_FILE_SIZE:u64 = 250;



fn main() {
	update_task_handler_implementations();
}



fn update_task_handler_implementations() {
	if !GENERATED_HANDLER_IMPLEMENTATIONS_FILE.exists() || !GENERATED_HANDLER_TESTS_FILE.exists() || GENERATED_HANDLER_IMPLEMENTATIONS_FILE.bytes_size() < GENERATED_HANDLER_MIN_FILE_SIZE || GENERATED_HANDLER_TESTS_FILE.bytes_size() < GENERATED_HANDLER_MIN_FILE_SIZE {
		generate_task_handler_implementations();
	}
}

fn generate_task_handler_implementations() {
	const IMPLEMENTATION_PREFIX:&str = r#"
		use crate::{ Event, TaskHandler as TH, TaskHandlerSource as THC };
		use std::{ error::Error, sync::Arc };
		
		fn bih<T:THC>(t:T) -> TH { t.into_handler() }
	"#;
	const TESTS_PREFIX:&str = r#"
		#[cfg(test)]
		mod tests {
			use crate::{ TaskHandlerSource, Event, TaskHandler };
			use std::{ error::Error, sync::{ Arc, Mutex } };

	"#;
	const TESTS_SUFFIX:&str = r#"
		}
	"#;
	
	let mut implementation_content:String = String::new();
	let mut tests_content:String = String::new();
	handler_implementations_for_all_singular_types(&mut implementation_content, &mut tests_content);
	handler_implementations_for_all_sets(&mut implementation_content, &mut tests_content);

	GENERATED_HANDLER_IMPLEMENTATIONS_FILE.write(untab_str(IMPLEMENTATION_PREFIX) + "\n\n\n" + &implementation_content).expect("Could not write generated implementations to file.");
	GENERATED_HANDLER_TESTS_FILE.write(untab_str(TESTS_PREFIX) + "\n\n\n" + &tests_content + &untab_str(TESTS_SUFFIX)).expect("Could not write generated implementations tests to file.");
}



fn handler_implementations_for_all_singular_types(implementation_content:&mut String, test_content:&mut String) {
	const WRAPPERS:&[[&str; 2]] = &[["Box", "FnMut"], ["Box", "Fn"], ["Arc", "Fn"]];
	const FN_ARGS:&[&[[&str; 2]]] = &[&[], &[["event", "&mut Event"]]];
	const RETURN_TYPES:&[&str] = &["", "-> Result<(), Box<dyn Error>>"];
	const INPUT_ARGS:&[[&str; 2]] = &[["event", "&mut Event"]];

	*implementation_content += "\n\n\n";
	for [wrapper, fn_type] in WRAPPERS {
		for fn_arg_set in FN_ARGS {
			for return_type in RETURN_TYPES {
				handler_implementation_for_singular_type(implementation_content, test_content, INPUT_ARGS, *wrapper, *fn_type, *fn_arg_set, *return_type);
			}
		}
	}
}

fn handler_implementation_for_singular_type(implementation_content:&mut String, test_content:&mut String, input_args:&[[&str; 2]], wrapper:&str, fn_type:&str, fn_arg_set:&[[&str; 2]], return_type:&str) {
	const IMPLEMENTATION_TEMPLATE:&str = r#"
		impl THC for $wrapper<dyn $fn_type($fn_arg_types) $return_type + Send + Sync + 'static> {
			fn into_handler($self) -> TH {
				TH::$task_handler_type(Box::new(move |$input_args| { self($fn_arg_names)$return_value }))
			}
		}
	"#;
	const TEST_TEMPLATE:&str = r#"
		#[test]
		fn handler_source_$test_name_suffix_specified_source() {
			static RUN_PROOF:Mutex<u8> = Mutex::new(0);
			$owned_var_definition
			let handler_source:$wrapper<dyn $fn_type($fn_arg_types) $return_type + Send + Sync + 'static> = $wrapper::new(
				$fn_move|$fn_arg_names| {
					$fn_body
				}
			);
			let mut handler:TaskHandler = handler_source.into_handler();
			let mut event:Event = Event::default();
			
			for index in 1..=64 {
				handler.run(&mut event).unwrap();
				assert_eq!(*RUN_PROOF.lock().unwrap(), index);
				assert_eq!(event.expired, $expired_check);
			}
		}
	"#;
	const EXPIRE_AT:&str = "50";

	let input_args_str:String = input_args.iter().map(|input_arg| if fn_arg_set.contains(input_arg) { input_arg[0].to_string() } else { "_".to_string() + input_arg[0] }).collect::<Vec<String>>().join(", ");
	let fn_arg_names:String = fn_arg_set.iter().map(|[arg_name, _]| *arg_name).collect::<Vec<&str>>().join(", ");
	let fn_arg_types:String = fn_arg_set.iter().map(|[_, arg_type]| *arg_type).collect::<Vec<&str>>().join(", ");
	let is_mut:bool = fn_type.contains("Mut");
	let has_return_type:bool = return_type != "";
	let has_event:bool = fn_arg_names.contains("event");

	if !(wrapper == "Box" && fn_arg_set.len() == input_args.len() && has_return_type) {
		*implementation_content += &apply_template(IMPLEMENTATION_TEMPLATE, &[
			("wrapper", wrapper),
			("fn_type", fn_type),
			("input_args", &input_args_str),
			("fn_arg_types", &fn_arg_types),
			("fn_arg_names", &fn_arg_names),
			("return_type", return_type),
			("self", if is_mut { "mut self" } else { "self" }),
			("task_handler_type", fn_type),
			("return_value", if has_return_type { "" } else { "; Ok(())" })
		]);
	}

	*test_content += &(tab_str(&apply_template(TEST_TEMPLATE, &[
		("test_name_suffix", &format!("{}_{}_{}args_{}_result", wrapper.to_lowercase(), fn_type.to_lowercase(), fn_arg_set.len(), if return_type == "" { "no" } else { "has" })),
		("owned_var_definition", if is_mut { "\n\tlet mut owned_index:u8 = 0;" } else { "" }),
		("wrapper", wrapper),
		("fn_type", fn_type),
		("input_args", &input_args_str),
		("fn_arg_types", &fn_arg_types),
		("fn_arg_names", &fn_arg_names),
		("return_type", return_type),
		("fn_move", if is_mut { "move " } else { "" }),
		("fn_body", &[
			if is_mut { "owned_index += 1;\n\t\t\t*RUN_PROOF.lock().unwrap() = owned_index;" } else { "*RUN_PROOF.lock().unwrap() += 1;" },
			&if has_event { "if *RUN_PROOF.lock().unwrap() == ".to_string() + EXPIRE_AT + " {{ event.expire(); }}" } else { String::new() },
			if has_return_type { "Ok(())" } else { "" }
		].iter().filter(|line| !line.is_empty()).map(|line| line.to_string()).collect::<Vec<String>>().join("\n\t\t\t")),
		("expired_check", &if has_event { "index >= ".to_string() + EXPIRE_AT } else { "false".to_string() })
	]))+ "\n");
}

fn handler_implementations_for_all_sets(implementation_content:&mut String, test_content:&mut String) {
	*implementation_content += "\n\n\n";
	for length in 2..64 {
		handler_implementation_for_set(implementation_content, test_content, length);
	}
}

fn handler_implementation_for_set(implementation_content:&mut String, test_content:&mut String, set_length:usize) {
	const IMPLEMENTATION_TEMPLATE:&str = r#"
		impl<$generic_definition> THC for ($set_identification) {
			fn into_handler(self) -> TH {
				TH::List((vec![$list_contents], 0))
			}
		}
	"#;
	const TEST_TEMPLATE:&str = r#"
		#[test]
		fn handler_source_$test_name_suffix() {
			static RUN_PROOF:Mutex<u8> = Mutex::new(0);
			$owned_var_definition
			let handler_source:$wrapper<dyn $fn_type($fn_arg_types) $return_type + Send + Sync + 'static> = $wrapper::new(
				$fn_move|$fn_arg_names| {
					$fn_body
				}
			);
			let mut handler:TaskHandler = handler_source.into_handler();
			let mut event:Event = Event::default();
			
			for index in 1..=64 {
				handler.run(&mut event).unwrap();
				assert_eq!(*RUN_PROOF.lock().unwrap(), index);
				assert_eq!(event.expired, $expired_check);
			}
		}
	"#;

	let generic_names:Vec<String> = (0..set_length).map(|length| {
		if length < 26 {
			[('A' as u8 + length as u8) as char].iter().collect()
		} else {
			[('A' as u8 + ((length / 26).max(1) - 1) as u8) as char, ('A' as u8 + (length % 26) as u8) as char].iter().collect()
		}
	}).collect::<Vec<String>>();

	*implementation_content += &apply_template(IMPLEMENTATION_TEMPLATE, &[
		("generic_definition", &generic_names.iter().map(|generic_name| format!("{generic_name}:THC + 'static")).collect::<Vec<String>>().join(", ")),
		("set_identification", &generic_names.join(", ")),
		("list_contents", &(0..set_length).map(|index| format!("bih(self.{index})")).collect::<Vec<String>>().join(", "))
	]);

	//*test_content += &apply_template(TEST_TEMPLATE, &[
	//	("set_{}len", &set_length.to_string())
	//]);
}



fn apply_template(template:&str, variables:&[(&str, &str)]) -> String {
	let mut content:String = untab_str(template);
	for (name, value) in variables {
		content = content.replace(&format!("${name}"), value);
	}
	content + "\n"
}
fn untab_str(source_str:&str) -> String {
	let lines:Vec<&str> = source_str.split('\n').skip_while(|line| line.chars().all(|character| character.is_whitespace())).collect();
	let initial_padding:String = lines.first().map(|line| line.chars().take_while(|character| character.is_whitespace()).collect::<String>()).unwrap_or_default();
	lines.into_iter().map(|line| if line.starts_with(&initial_padding) { &line[initial_padding.len()..] } else { line }).collect::<Vec<&str>>().join("\n").trim_end().to_string()
}
fn tab_str(source_str:&str) -> String {
	source_str.split('\n').map(|line| "\t".to_string() + line).collect::<Vec<String>>().join("\n")
}