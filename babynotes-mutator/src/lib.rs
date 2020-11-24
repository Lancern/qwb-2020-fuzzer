extern crate bincode;
extern crate fuzzer;

use std::os::raw::{c_uint, c_void};

use fuzzer::{Command, CommandDataSpec, CommandSpec, Fuzzer, FuzzerBuilder, Input};

#[allow(non_camel_case_types)]
type afl_t = c_void;

const CMD_ADD_NOTE: i32 = 1;
const CMD_SHOW_NOTE: i32 = 2;
const CMD_DELETE_NOTE: i32 = 3;
const CMD_EDIT_NOTE: i32 = 4;
const CMD_RESET: i32 = 5;
const CMD_CHECK: i32 = 6;
const CMD_EXIT: i32 = 7;

#[no_mangle]
pub extern "C" fn afl_custom_init(afl: *const afl_t, seed: c_uint) -> *const c_void {
    let f = FuzzerBuilder::new(afl, seed as u32)
        .add_spec(CommandSpec {
            id: CMD_ADD_NOTE,
            data: vec![
                CommandDataSpec::UInt { min: 0, max: 5 },
                CommandDataSpec::UInt { min: 0, max: 256 },
            ],
        })
        .add_spec(CommandSpec {
            id: CMD_SHOW_NOTE,
            data: vec![CommandDataSpec::UInt { min: 0, max: 5 }],
        })
        .add_spec(CommandSpec {
            id: CMD_DELETE_NOTE,
            data: vec![CommandDataSpec::SInt {
                min: std::i64::MIN,
                max: 3,
            }],
        })
        .add_spec(CommandSpec {
            id: CMD_EDIT_NOTE,
            data: vec![
                CommandDataSpec::UInt { min: 0, max: 3 },
                CommandDataSpec::Binary {
                    min_len: 1,
                    max_len: 512,
                },
            ],
        })
        .add_spec(CommandSpec {
            id: CMD_RESET,
            data: vec![],
        })
        .add_spec(CommandSpec {
            id: CMD_CHECK,
            data: vec![],
        })
        .add_spec(CommandSpec {
            id: CMD_EXIT,
            data: vec![],
        })
        .build();
    Box::into_raw(f) as *const c_void
}

#[no_mangle]
pub extern "C" fn afl_custom_fuzz(
    data: *const c_void,
    buf: *const u8,
    buf_size: usize,
    out_buf: *mut *const u8,
    _add_buf: *const u8,
    _add_buf_size: usize,
    _max_size: usize,
) -> usize {
    let f = as_fuzzer(data);
    let buf = unsafe { std::slice::from_raw_parts(buf, buf_size) };
    let mut input = bincode::deserialize::<Input>(buf).expect("deserialize input failed");

    f.mutate(&mut input);

    bincode::serialize_into(f.alloc_buf(), &input).expect("serialize input failed");

    unsafe {
        *out_buf = f.get_buf().as_ptr();
    }
    f.get_buf().len()
}

#[no_mangle]
pub extern "C" fn afl_custom_post_process(
    data: *const c_void,
    buf: *const u8,
    buf_size: usize,
    out_buf: *mut *const u8,
) -> usize {
    let f = as_fuzzer(data);
    let buf = unsafe { std::slice::from_raw_parts(buf, buf_size) };
    let mut input = bincode::deserialize::<Input>(buf).expect("deserialize input failed");

    input.commands.push(Command {
        id: CMD_EXIT,
        data: vec![],
    });

    input
        .synthesis_into(f.alloc_buf())
        .expect("synthesis input failed");

    unsafe {
        *out_buf = f.get_buf().as_ptr();
    }
    f.get_buf().len()
}

#[no_mangle]
pub extern "C" fn afl_custom_deinit(data: *const c_void) {
    unsafe { Box::from_raw(data as *mut Fuzzer) };
}

fn as_fuzzer<'a>(ptr: *const c_void) -> &'a mut Fuzzer {
    unsafe { (ptr as *mut Fuzzer).as_mut().expect("null fuzzer pointer") }
}
