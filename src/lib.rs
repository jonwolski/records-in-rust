#[derive(Copy, Clone)]
pub struct Record {
    a: u32,
    b: u32,
    c: bool,
}

// funcitonal/immutable record style

fn get_toggled_record(record: Record) -> Record {
    Record {
        c: !record.c,
        ..record
    }
}

fn get_incremented_record(record: Record) -> Record {
    Record {
        a: record.a + 1,
        ..record
    }
}

fn get_accumulated_record(record: Record) -> Record {
    Record {
        a: record.a + record.b,
        b: record.a,
        ..record
    }
}

// mutate-in-place style

fn mut_toggled_record(record: &mut Record) {
    record.c = !record.c;
}

fn mut_incremented_record(record: &mut Record) {
    record.a = record.a + 1;
}

fn mut_accumulated_record(record: &mut Record) {
    record.a = record.a + record.b;
    record.b = record.a;
}

/// mutate in-place, imperative style
#[inline(never)]
pub fn update_record_with_refs(record: &mut Record) {
    mut_toggled_record(record);
    mut_incremented_record(record);
    mut_accumulated_record(record);
}

/// immutable record, functional style
#[inline(never)]
pub fn update_record_with_ptrs(record: &mut Record) {
    *record = get_toggled_record(*record);
    *record = get_incremented_record(*record);
    *record = get_accumulated_record(*record);
}

/// minimize use of pointers by nesting function calls
#[inline(never)]
pub fn update_record_with_minimal_vars(record: &mut Record) {
    *record = get_accumulated_record(get_incremented_record(get_toggled_record(*record)));
}

/// minimize use of pointers with shadowed tmp vars
#[inline(never)]
pub fn update_record_with_shadowed_vars(record: &mut Record) {
    let tmp = *record;
    let tmp = get_toggled_record(tmp);
    let tmp = get_incremented_record(tmp);
    let tmp = get_accumulated_record(tmp);
    *record = tmp;
}

/// minimize use of pointers with a single, mutable tmp var
#[inline(never)]
pub fn update_record_with_mut_tmp_var(record: &mut Record) {
    let mut tmp = *record;
    tmp = get_toggled_record(tmp);
    tmp = get_incremented_record(tmp);
    tmp = get_accumulated_record(tmp);
    *record = tmp;
}

#[inline(never)]
pub fn update_record_no_refs(record: Record) -> Record {
    let mut record = get_toggled_record(record);
    record = get_incremented_record(record);
    record = get_accumulated_record(record);
    record
}

#[inline(never)]
pub fn update_record_mut(record: Record) -> Record {
    let mut record = record;
    mut_toggled_record(&mut record);
    mut_incremented_record(&mut record);
    mut_accumulated_record(&mut record);
    record
}
