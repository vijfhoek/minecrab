/// Returns `x` incremented with the lowest possible value that a
/// single-precision floating point with `x`'s value can represent.
pub fn f32_successor(x: f32) -> f32 {
    let x = x.to_bits();
    let x = if (x >> 31) == 0 { x + 1 } else { x - 1 };
    f32::from_bits(x)
}

/// Returns `x` decremented with the lowest possible value that a
/// single-precision floating point with `x`'s value can represent.
pub fn f32_predecessor(x: f32) -> f32 {
    let x = x.to_bits();
    let x = if (x >> 31) == 0 { x - 1 } else { x + 1 };
    f32::from_bits(x)
}
