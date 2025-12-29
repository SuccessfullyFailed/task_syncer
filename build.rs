use file_ref::FileRef;



const GENERATED_HANDLER_IMPLEMENTATIONS_FILE:FileRef = FileRef::new_const("src/task_handler_generated_implementations.rs");
const GENERATED_HANDLER_MIN_FILE_SIZE:u64 = 250;



fn main() {
	update_task_handler_implementations();
}



fn update_task_handler_implementations() {
	if !GENERATED_HANDLER_IMPLEMENTATIONS_FILE.exists() || GENERATED_HANDLER_IMPLEMENTATIONS_FILE.bytes_size() < GENERATED_HANDLER_MIN_FILE_SIZE {
		generate_task_handler_implementations();
	}
}

fn generate_task_handler_implementations() {
	const PREFIX:&str = r#"
		use crate::{ Event, TaskHandler as TH, TaskHandlerSource as THC };
		use std::{ error::Error, sync::Arc };
		
		fn ih<T:THC>(t:T) -> TH { t.into_handler() }
	"#;
	
	let output_content:String = format!("{}\n\n\n\n{}\n\n\n\n{}\n\n\n\n{}", untab_str(PREFIX), handler_implementations_for_all_singular_types(), handler_implementations_for_all_lists(), handler_implementations_for_all_sets());
	GENERATED_HANDLER_IMPLEMENTATIONS_FILE.write(output_content).expect("Could not write generated implementations to file.");
}

fn handler_implementations_for_all_singular_types() -> String {
	const WRAPPERS:&[[&str; 2]] = &[["Box", "FnMut"], ["Arc", "Fn"]];
	const FN_ARGS:&[&[[&str; 2]]] = &[&[], &[["event", "&mut Event"]]];
	const RETURN_TYPES:&[&str] = &["", "-> Result<(), Box<dyn Error>>"];
	const INPUT_ARGS:&[[&str; 2]] = &[["event", "&mut Event"]];

	let mut output_content:String = String::new();
	for [wrapper, fn_type] in WRAPPERS {
		for fn_arg_set in FN_ARGS {
			for return_type in RETURN_TYPES {
				output_content += &(handler_implementation_for_singular_type(INPUT_ARGS, *wrapper, *fn_type, *fn_arg_set, *return_type) + "\n");
			}
		}
	}
	output_content
}

fn handler_implementation_for_singular_type(input_args:&[[&str; 2]], wrapper:&str, fn_type:&str, fn_arg_set:&[[&str; 2]], return_type:&str) -> String {
	const TEMPLATE:&str = r#"
		impl THC for $wrapper<dyn $fn_type($fn_args) $return_type + Send + Sync + 'static> {
			fn into_handler($self) -> TH {
				TH::$task_handler_type($implementation)
			}
		}
	"#;
	let template:String = untab_str(TEMPLATE);

	let fn_arg_names:Vec<&str> = fn_arg_set.iter().map(|[arg_name, _]| *arg_name).collect();
	let fn_arg_types:Vec<&str> = fn_arg_set.iter().map(|[_, arg_type]| *arg_type).collect();
	template
		.replace("$wrapper", wrapper)
		.replace("$fn_type", fn_type)
		.replace("$fn_args", &fn_arg_types.join(", "))
		.replace("$return_type", return_type)
		.replace("$self", if fn_type.contains("Mut") { "mut self" } else { "self" })
		.replace("$task_handler_type", fn_type)
		.replace("$implementation", &format!(
			"Box::new(move |{}| {{ self({}){} }})",
			input_args.iter().map(|input_arg| if fn_arg_set.contains(input_arg) { input_arg[0].to_string() } else { "_".to_string() + input_arg[0] }).collect::<Vec<String>>().join(", "),
			fn_arg_names.join(", "),
			if return_type == "" { "; Ok(())" } else { "" }
		))
}

fn handler_implementations_for_all_lists() -> String {
	untab_str(
		r#"
			impl<T:THC + Clone + 'static, const SIZE:usize> THC for [T; SIZE] {
				fn into_handler(self) -> TH {
					self.to_vec().into_handler()
				}
			}
			impl<T:THC + 'static> THC for Vec<T> {
				fn into_handler(self) -> TH {
					TH::List((self.into_iter().map(|source| source.into_handler()).collect(), 0))
				}
			}
		"#
	)
}


fn handler_implementations_for_all_sets() -> String {
	(2..64).map(|length| handler_implementation_for_set(length)).collect::<Vec<String>>().join("\n")
}
fn handler_implementation_for_set(set_length:usize) -> String {
	const TEMPLATE:&str = r#"
		impl<$generic_definition> THC for ($set_identification) {
			fn into_handler(self) -> TH {
				TH::List((vec![$list_contents], 0))
			}
		}
	"#;
	let template:String = untab_str(TEMPLATE);

	let generic_names:Vec<String> = (0..set_length).map(|length| {
		if length < 26 {
			[('A' as u8 + length as u8) as char].iter().collect()
		} else {
			[('A' as u8 + ((length / 26).max(1) - 1) as u8) as char, ('A' as u8 + (length % 26) as u8) as char].iter().collect()
		}
	}).collect::<Vec<String>>();
	
	template
		.replace("$generic_definition", &generic_names.iter().map(|generic_name| format!("{generic_name}:THC + 'static")).collect::<Vec<String>>().join(", "))
		.replace("$set_identification", &generic_names.join(", "))
		.replace("$list_contents", &(0..set_length).map(|index| format!("ih(self.{index})")).collect::<Vec<String>>().join(", "))
}



fn untab_str(source_str:&str) -> String {
	let lines:Vec<&str> = source_str.split('\n').skip_while(|line| line.chars().all(|character| character.is_whitespace())).collect();
	let initial_padding:String = lines.first().map(|line| line.chars().take_while(|character| character.is_whitespace()).collect::<String>()).unwrap_or_default();
	lines.into_iter().map(|line| if line.starts_with(&initial_padding) { &line[initial_padding.len()..] } else { line }).collect::<Vec<&str>>().join("\n").trim_end().to_string()
}