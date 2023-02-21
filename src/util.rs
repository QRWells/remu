pub(crate) fn float_classify(x: f32) -> u64 {
    let mut res = 0;
    if x == f32::NEG_INFINITY {
        res |= 1;
    }
    if x.is_normal() && x.is_sign_negative() {
        res |= 2;
    }
    if x.is_subnormal() && x.is_sign_negative() {
        res |= 4;
    }
    if x == 0.0 && x.is_sign_negative() {
        res |= 8;
    }
    if x == 0.0 && x.is_sign_positive() {
        res |= 16;
    }
    if x.is_subnormal() && x.is_sign_positive() {
        res |= 32;
    }
    if x.is_normal() && x.is_sign_positive() {
        res |= 64;
    }
    if x == f32::INFINITY {
        res |= 128;
    }
    if x.is_nan() && !quiet_nan(x) {
        res |= 256;
    }
    if x.is_nan() && quiet_nan(x) {
        res |= 512;
    }
    res
}

pub(crate) fn quiet_nan(value: f32) -> bool {
    let bits = value.to_bits();
    (bits & 0x7f800000) == 0x7f800000 && (bits & 0x007fffff) != 0
}

pub(crate) fn double_classify(x: f64) -> u64 {
    let mut res = 0;
    if x == f64::NEG_INFINITY {
        res |= 1;
    }
    if x.is_normal() && x.is_sign_negative() {
        res |= 2;
    }
    if x.is_subnormal() && x.is_sign_negative() {
        res |= 4;
    }
    if x == 0.0 && x.is_sign_negative() {
        res |= 8;
    }
    if x == 0.0 && x.is_sign_positive() {
        res |= 16;
    }
    if x.is_subnormal() && x.is_sign_positive() {
        res |= 32;
    }
    if x.is_normal() && x.is_sign_positive() {
        res |= 64;
    }
    if x == f64::INFINITY {
        res |= 128;
    }
    if x.is_nan() && !quiet_nan_double(x) {
        res |= 256;
    }
    if x.is_nan() && quiet_nan_double(x) {
        res |= 512;
    }
    res
}

pub(crate) fn quiet_nan_double(value: f64) -> bool {
    let bits = value.to_bits();
    (bits & 0x7ff0000000000000) == 0x7ff0000000000000 && (bits & 0x000fffffffffffff) != 0
}

pub(crate) fn addr_add(addr: u64, offset: i32) -> u64 {
    if offset.is_negative() {
        addr - offset.wrapping_abs() as u32 as u64
    } else {
        addr + offset as u64
    }
}
