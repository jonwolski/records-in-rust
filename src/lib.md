# Zero-cost Functional Records in Rust

A friend and colleague piqued my interest with a comment about using an
immutable functional style in Haskell or Rust and the invisible impact it has
on performance compared ostensibly to imperative C or C++.

I've played with Haskell only enough to "learn me one for great good"—not
enough to know about any performance gotchas. However, I _do_ have some Rust
in production, so I'll focus on that.

<aside style="border: 0.3ex solid; border-radius: 1ex; padding: 1em; margin: 1.5em;">

**tl;dr: Rust enables zero-cost functional style<sup>[1][1]</sup>**
</aside>

Let's see what Rust can do.

First lets create a `struct`. I'll call my new datatype `Record`. (This is
just my name for it; "Record" is not a Rust keyword.)

```rust
#[derive(Copy, Clone)]
pub struct Record {
    a: u32,
    b: u32,
    c: bool,
}
```

To make it interesting, I've given the `Record` type some fields arbitrarily:

* `a` - an unsigned 32-bit integer (`u32`)
* `b` - an unsigned 32-bit integer (`u32`)
* `c` - a Boolean (`bool`)

Now let's create some functions to mutate the struct.

## Mutating the fields in-place

Ostensibly, mutating the record in-place will be more 'performant' than creating a copy of the struct each time we want to change a field.

### Toggle the Record's Boolean flag

```rust
fn toggle_record(record: &mut Record) {
    record.c = !record.c;
}
```

For those unfamiliar with Rust syntax, this function takes a _mutable_
reference to a Record (`&mut Record`) and binds it to a parameter named
`record`. The function updates the record's `c` field. De-referencing is
implicit.

It returns `Void`.

Let's have a couple more functions to update the other fields.

```rust
fn increment_record(record: &mut Record) {
    record.a = record.a + 1;
}

/// this will treat `a` as an "accumulator"
/// Mostly I wrote this function just to give the compiler some more
/// difficulty by both reading from and writing to the same variable.
fn accumulate_record(record: &mut Record) {
    record.a = record.a + record.b;
    record.b = record.a;
}
```

## Functional / Immutable Style

In functional programming, a function has no side-effects. Rather than mutate state, the function returns _new_ state.

```rust
#[allow(dead_code)]
fn get_toggled_record_by_copying(record: Record) -> Record {
    // Record { ... }  is the syntax for creating a new Record
    Record {
        a: !record.a,
        b: record.b,
        c: record.c,
    }
}
```

Copying _some_ fields while replacing others comes up enough that there is
a simplified syntax for it called ["struct update syntax."][5.1]

[5.1]: https://doc.rust-lang.org/book/ch05-01-defining-structs.html#creating-instances-from-other-instances-with-struct-update-syntax

```rust
fn get_toggled_record(record: Record) -> Record {
    Record {
        c: !record.c,
        ..record
    }
}
```

Here are the remaining functions from before, rewritten in functional/immutable
style with "struct update syntax."

```rust
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
```

## Testing the approaches

Now that I have a couple different ways of writing these functions—an
imperative style and a functional style—I will write some client code to call
these functions, and then compare their assembly code.

### Methodology

A debug build will result in very different compilation for these functions,
but I am interested in whether or not Rust can 'magically' optimize away my
extra struct creations.

For these tests, I will do a `cargo rustc --release -- --emit asm`.

To prevent the compiler from optimizing away nearly all the code, I have
created a 'lib' project instead of a 'bin' (something with a `main.rs`).

Additionally, I will use `#[inline(never)]` on the calling functions so that I can compare the compilations of different ways of calling this code.

### First up: imperative style

Probably the most intuitive approach and the most C-like. Pass a structure by
(mutable) reference, mutate the structure, return nothing.

```rust
/// mutate in-place, imperative style
#[inline(never)]
pub fn update_record_with_refs(record: &mut Record) {
    toggle_record(record);
    increment_record(record);
    accumulate_record(record);
}
```

Let's view the corresponding assembly:

```sh
# target arch and compiler version
uname -v ;\
cargo --version ;\
cargo rustc --release -- --emit asm &&\
find target -name '*.s' -exec cat {} \;
```

The salient parts:

```
Darwin Kernel Version 24.5.0: Tue Apr 22 19:48:46 PDT 2025; root:xnu-11417.121.6~2/RELEASE_ARM64_T8103
cargo 1.86.0 (adf9b6ad1 2025-02-28)
...
```

```asm
__ZN15records_in_rust23update_record_with_refs17h80c99250a4a3f79fE:
	.cfi_startproc
	ldrb	w8, [x0, #8]
	eor	w8, w8, #0x1
	strb	w8, [x0, #8]
	ldp	w8, w9, [x0]
	add	w8, w8, w9
	add	w8, w8, #1
	stp	w8, w8, [x0]
	ret
	.cfi_endproc
```

I will not dig too deep on understanding the assembly—I am more interested in
_comparing_ the assembly of the different functions—but at first glance, this
looks like the following:

1. load a byte from memory (`ldrb`) into register `w8`
2. XOR (`eor`) the register with `1`. I.e. toggle the boolean
3. store the modified byte back (`strb`)
4. read two (32-bit) ints into registers `w8`, and `w9` (`ldp ...`)
5. `add` those ints together
6. `add` `1` to one of them
7. write both registers back to memory (`stp`)
8. `ret`-urn from the procedure

This maps pretty closely to the imperative functions being called.

### Next up, immutable / functional style

Now to call the non-mutating functions that _appear_ to create copies of the
data.

Immediately, I run into a dilemma. I want to test whether these functions will
create a copy of the data or optimize by mutating-in-place. I will

I can't just create a record and pass it in to the functions under test,
because the optimizer will eliminate the code. I need to leave something open-ended for _run-time_; it needs to be an _input_ to my library.

However, if I want to give the compiler the opportunity to optimize this "copy"
as an in-place mutation, I need mutable memory.

Since I'm testing the immutable-style functions, I will allow my _client_
functions (those functions _executing the test_) to receive a mutable
reference. My functions _under test_ will still receive immutable structs and
return copies.

```rust
/// immutable record, functional style
#[inline(never)]
pub fn update_record_with_ptrs(record: &mut Record) {
    *record = get_toggled_record(*record);
    *record = get_incremented_record(*record);
    *record = get_accumulated_record(*record);
}
```

… and the corresponding assembly:

```asm
__ZN15records_in_rust23update_record_with_ptrs17hc5c1df94ab26400bE:
	.cfi_startproc
	ldrb	w8, [x0, #8]
	eor	w8, w8, #0x1
	ldp	w9, w10, [x0]
	add	w9, w9, #1
	add	w10, w9, w10
	stp	w10, w9, [x0]
	strb	w8, [x0, #8]
	ret
	.cfi_endproc
```
Whadya know? It's nearly identical code! The `strb` is moved to the end,
but that does not make much difference.

Okay, but I would never write functional code that way. When I'm "functional
programming" I try to live in [a world without assignment][awwoa].

To the extend that Rust allows, let's mimic what I might do in Haskell.

(Since the test function receives a record and returns nothing, I will still
need one assignment at the end of the call chain.)

[awwoa]: https://www.rubyevents.org/talks/a-world-without-assignment

```rust
/// minimize use of pointers by nesting function calls
#[inline(never)]
pub fn update_record_with_minimal_vars(record: &mut Record) {
*record = get_accumulated_record(get_incremented_record(get_toggled_record(*record)));
}
```

That produces ...

```asm
__ZN15records_in_rust31update_record_with_minimal_vars17h1bc5b1920fe2a2daE:
	.cfi_startproc
	ldp	w8, w9, [x0]
	ldrb	w10, [x0, #8]
	eor	w10, w10, #0x1
	add	w8, w8, #1
	add	w9, w8, w9
	stp	w9, w8, [x0]
	strb	w10, [x0, #8]
	ret
	.cfi_endproc
```

Again the same assembly in a slightly different order.

Still, this is kind of ugly. What happens if I save the intermediate values to
a temp var. I'll re-use the same var name, _shadowing_ the previous var each
time.

```rust
/// minimize use of pointers with shadowed tmp vars
#[inline(never)]
pub fn update_record_with_shadowed_vars(record: &mut Record) {
    let tmp = *record;
    let tmp = get_toggled_record(tmp);
    let tmp = get_incremented_record(tmp);
    let tmp = get_accumulated_record(tmp);
    *record = tmp;
}
```

… compile …

```asm
__ZN15records_in_rust30update_record_with_mut_tmp_var17hb6e407b831178dc9E:
	.cfi_startproc
	ldp	w8, w9, [x0]
	ldrb	w10, [x0, #8]
	mov	w11, #1
	bic	w10, w11, w10
	add	w8, w8, #1
	add	w9, w8, w9
	stp	w9, w8, [x0]
	strb	w10, [x0, #8]
	ret
	.cfi_endproc
```

Oddly, the only difference is the way the toggle is performed.

```asm
mov	w11, #1             ; w11 = 1
bic	w10, w11, w10       ; w10 = w11 & ~w10
```

That's an unexpected way to toggle a boolean. I don't really know what to say
about that.

...

Well, on to the next thing. What happens if I re-use a temp var instead of
shadowing the previous var?


```rust
/// minimize use of pointers with a single, mutable tmp var
#[inline(never)]
pub fn update_record_with_mut_tmp_var(record: &mut Record) {
    let mut tmp = *record;
    tmp = get_toggled_record(tmp);
    tmp = get_incremented_record(tmp);
    tmp = get_accumulated_record(tmp);
    *record = tmp;
}
```

```asm
__ZN15records_in_rust30update_record_with_mut_tmp_var17hb6e407b831178dc9E:
	.cfi_startproc
	ldp	w8, w9, [x0]
	ldrb	w10, [x0, #8]
	mov	w11, #1
	bic	w10, w11, w10
	add	w8, w8, #1
	add	w9, w8, w9
	stp	w9, w8, [x0]
	strb	w10, [x0, #8]
	ret
	.cfi_endproc
```

Same thing. No surprised there, but it's good to know.

### Can I do this with no refs?

At this point, it dawns on me my _test_ code could receive a `Record` rather
a reference to one. This _moves_ the struct into our function (rather than a
_borrowing_ the struct).

Perhaps we stay truer to functional style by moving a `record` into the
function, consuming it, and returning a new `Record`.

Again, I'll re-use a mutable var instead of shadowing, since that appeared to
not make a difference before.

```rust
#[inline(never)]
pub fn update_record_no_refs(record: Record) -> Record {
    let mut record = get_toggled_record(record);
    record = get_incremented_record(record);
    record = get_accumulated_record(record);
    record
}
```

Compile ...

```asm
__ZN15records_in_rust21update_record_no_refs17h6f48abe941590117E:
	.cfi_startproc
	ldp	w9, w10, [x0]
	ldrb	w11, [x0, #8]
	eor	w11, w11, #0x1
	add	w9, w9, #1
	add	w10, w9, w10
	stp	w10, w9, [x8]
	strb	w11, [x8, #8]
	ret
	.cfi_endproc
```

Same as the earlier code, but we're back to using `XOR` (`eor`) to toggle
the Boolean.

Finally, let's see what happens when my test code receives a `Record` and
returns a new `Record`, but I call the mutating/imperative functions.

```rust
#[inline(never)]
pub fn update_record_mut(record: Record) -> Record {
    // Re-bind as mutable. This is legal because
    // at this point the function owns `record`
    let mut record = record;
    toggle_record(&mut record);
    increment_record(&mut record);
    accumulate_record(&mut record);
    record
}
```

```asm
__ZN15records_in_rust17update_record_mut17h90ee354aab9a591eE:
	.cfi_startproc
	ldp	w9, w10, [x0]
	ldrb	w11, [x0, #8]
	ldurh	w12, [x0, #9]
	sturh	w12, [x8, #9]
	ldrb	w12, [x0, #11]
	strb	w12, [x8, #11]
	mov	w12, #1
	bic	w11, w12, w11
	add	w9, w9, w10
	add	w9, w9, #1
	stp	w9, w9, [x8]
	strb	w11, [x8, #8]
	ret
	.cfi_endproc
```

Whoa! For the first time, **a second struct is created!**

Also, this loading is weird because my struct is represented in 9 bytes. It doesn't align well to "word boundaries."

This is why we see `ldurh w12, [x0, #9]` ("load half-word starting at byte 9 into register `w12`").

We see loads from `x0`, but stores to `x8` indicating we are reading from one
struct and writing to a second.

The re-binding _should_ be a no-op, but I'll check anyway.

```rust
#[inline(never)]
pub fn update_mut_record_mut(mut record: Record) -> Record {
    toggle_record(&mut record);
    increment_record(&mut record);
    accumulate_record(&mut record);
    record
}
```

…

```asm

__ZN15records_in_rust21update_mut_record_mut17h40a89b93d70ddfc7E:
	.cfi_startproc
	ldrb	w9, [x0, #8]
	eor	w9, w9, #0x1
	strb	w9, [x0, #8]
	ldp	w9, w10, [x0]
	add	w9, w9, w10
	add	w9, w9, #1
	stp	w9, w9, [x0]
	ldr	w9, [x0, #8]
	str	w9, [x8, #8]
	ldr	x9, [x0]
	str	x9, [x8]
	ret
	.cfi_endproc
```

Different, but not significantly so.

## Conclusion

Rust (or LLVM) is able to optimize what appears to be "copy-construction" into
update-in-place when a function _consumes_ a struct and returns a copy of that struct, even with some modifications to the original struct.

Ironically, the one case where Rust still produced a _copy_ of the original struct was when I had to do a mutable re-binding.

---


[1]: #1
<footer id="1">

<sup>1</sup>Technically, it may be _LLVM_ that enables this, but I think the Rust compiler
team would have to do at least _some_ work to take advantage of it.

Also, when I say "zero cost," I mean "run-time performance cost." That is, writing in a functional style will not result in more memory usage or CPU cycles than writing in a more imperative, mutable style.
</footer>
