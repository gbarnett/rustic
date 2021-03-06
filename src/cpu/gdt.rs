/*
 * Copyright (c) 2013 Matthew Iselin
 *
 * Permission to use, copy, modify, and distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
 * WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
 * MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
 * ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
 * WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
 * ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
 * OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
 */

use core;
use core::mem::size_of;

type gdttable = [gdtentry, ..16];

#[packed]
struct gdtreg {
    limit: u16,
    addr: *gdttable,
}

#[packed]
struct gdtentry {
    limit_low: u16,
    base_low: u16,
    base_mid: u8,
    access: u8,
    gran: u8,
    base_high: u8,
}

struct table {
    reg: *mut gdtreg,
    table: *mut gdttable
}

impl gdtreg {
    pub fn new(gdt: *gdttable) -> gdtreg {
        gdtreg {
            addr: gdt,
            limit: (size_of::<gdttable>() + 1) as u16,
        }
    }
}

impl gdtentry {
    pub fn new(base: uint, limit: uint, access: u8, gran: u8) -> gdtentry {
        gdtentry {
            limit_low: (limit & 0xFFFF) as u16,
            base_low: (base & 0xFFFF) as u16,
            base_mid: ((base >> 16) & 0xFF) as u8,
            access: access,
            gran: gran,
            base_high: ((base >> 24) & 0xFF) as u8,
        }
    }
}

static mut systemgdt: table = table {
    table: 0 as *mut gdttable,
    reg: 0 as *mut gdtreg,
};

#[fixed_stack_segment]
pub fn init() {
    unsafe {
        systemgdt.table = core::heap::malloc(128) as *mut gdttable;
        systemgdt.reg = core::heap::malloc(6) as *mut gdtreg;
        *systemgdt.reg = gdtreg::new(systemgdt.table as *gdttable);
    }
}

pub fn load(codeseg: u16, dataseg: u16, tlsemulseg: u16) {
    unsafe { asm!(" \
        lgdt ($0); \
        jmp $1, $$.g; \
        .g: \
        mov $2, %ax; \
        mov %ax, %ds; \
        mov %ax, %es; \
        mov %ax, %fs; \
        mov %ax, %ss; \
        mov $3, %ax; \
        mov %ax, %gs;" :: "r" (systemgdt.reg), "Ir" (codeseg), "Ir" (dataseg), "Ir" (tlsemulseg) : "ax"); }
}

pub fn entry(index: int, base: uint, limit: uint, access: u8, gran: u8) {
    unsafe {
        (*systemgdt.table)[index] = gdtentry::new(base, limit, access, gran);
    }
}

