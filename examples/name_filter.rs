use proc_prog_name::ProcProgEntry;

const FILTER: &[&str] = &["proc_prog_name", "name_filter"];

fn main() {
    let entries = ProcProgEntry::get_all_entries_with_name_filter(FILTER);
    println!("{entries:#?}");
}
