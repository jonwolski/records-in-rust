//@ ---
//@ marp: true
//@ ---
//@
//@ # Let's define a record

#[derive(Copy, Clone)]
pub struct Record {
    a: u32,
    b: u32,
    c: bool,
}
//@ ---
//@
//@ ## Mutable update

fn toggle_record_flag(record: &mut Record) {
    record.c = !record.c;
}

//@ ---
//@
//@ ## Immutable "record-style" update

fn get_toggled_record(record: Record) -> Record {
    Record {
        c: !record.c,
        ..record
    }
}
