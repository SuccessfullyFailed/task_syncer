fn main() {
	crate_manager::generate_exports_for_crates_in_working_dir().expect("Could not generate auto-lib-exports.");
}