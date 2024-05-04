use proc_prog_name::ProcProgEntry;

fn main() {
    let entries = ProcProgEntry::get_all_proc_prog_entries();
    println!("{entries:#?}");
}
