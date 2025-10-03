#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]
#![crate_type = "cdylib"] 

use std::ffi as libc;

#[no_mangle]
pub unsafe extern "C" fn sub(
    mut vec0: *mut libc::c_float,
    mut vec1: *mut libc::c_float,
    mut vec2: *mut libc::c_float,
) {
    *vec0
        .offset(
            0 as libc::c_int as isize,
        ) = *vec1.offset(0 as libc::c_int as isize)
        - *vec2.offset(0 as libc::c_int as isize);
    *vec0
        .offset(
            1 as libc::c_int as isize,
        ) = *vec1.offset(1 as libc::c_int as isize)
        - *vec2.offset(1 as libc::c_int as isize);
    *vec0
        .offset(
            2 as libc::c_int as isize,
        ) = *vec1.offset(2 as libc::c_int as isize)
        - *vec2.offset(2 as libc::c_int as isize);
}
