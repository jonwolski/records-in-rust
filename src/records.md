---
marp: true
---

# Let's define a record

```rust
#[derive(Copy, Clone)]
pub struct Record {
    a: u32,
    b: u32,
    c: bool,
}
```
---

## Mutable update

```rust
fn toggle_record_flag(record: &mut Record) {
    record.c = !record.c;
}
```

---

## Immutable "record-style" update

```rust
fn get_toggled_record(record: Record) -> Record {
    Record {
        c: !record.c,
        ..record
    }
}
```