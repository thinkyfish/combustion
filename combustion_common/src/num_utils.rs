//! Numeric utilities

use std::ops::{Add, Mul};

use num_traits::{Num, Float};

/// Generic min function for any `PartialOrd`
///
/// ```
/// use combustion_common::num_utils::min;
///
/// assert_eq!(min(1, 2), 1);
/// ```
#[inline(always)]
pub fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b { a } else { b }
}

/// Generic max function for any `PartialOrd`
///
/// ```
/// use combustion_common::num_utils::max;
///
/// assert_eq!(max(1, 2), 2);
/// ```
#[inline(always)]
pub fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a >= b { a } else { b }
}

/// Generic min-max function for any `PartialOrd`
///
/// ```
/// use combustion_common::num_utils::min_max;
///
/// assert_eq!(min_max(1, 2), (1, 2));
/// ```
#[inline(always)]
pub fn min_max<T: PartialOrd>(a: T, b: T) -> (T, T) {
    if a >= b { (b, a) } else { (a, b) }
}

/// Round a number to a certain multiple
///
/// E.g.,
///
/// ```
/// use combustion_common::num_utils::round_multiple;
///
/// assert_eq!(round_multiple(43, 5), 45)
/// ```
#[inline(always)]
pub fn round_multiple<T: Num + Copy>(num: T, multiple: T) -> T {
    ((num + multiple - T::one()) / multiple) * multiple
}

/// Adds a `clamp` function to the type
///
/// E.g.,
///
/// ```
/// use combustion_common::num_utils::*;
///
/// assert_eq!(15u32.clamp(0, 5), 5);
/// assert!(3.14f32.clamp(0.0, 1.0) < 2.0);
/// assert!(0.4f32.clamp(1.5, 3.0) > 1.0)
/// ```
pub trait ClampExt {
    /// Clamps the value to `min` and `max` bounds.
    fn clamp(self, min: Self, max: Self) -> Self;
}

impl<T> ClampExt for T where T: PartialOrd {
    fn clamp(self, min: T, max: T) -> T {
        if self < min { min } else if self > max { max } else { self }
    }
}

/// Extension that provides approximate equality comparison for floating point numbers
///
/// E.g.,
///
/// ```
/// use combustion_common::num_utils::*;
///
/// assert!(5.12345f32.almost_eq(5.12, 0.1));
/// assert!(0.00000001f32.almost_eq(0.0, 0.0000001));
/// assert!(0.99999999f32.almost_eq(1.0, 0.0000001));
/// assert!(!(0.1.almost_eq(4.0, 0.1)));
/// ```
pub trait AlmostEqExt {
    /// Tests if two numbers are almost equal within a degree of accuracy
    ///
    /// E.g.:
    ///
    /// ```ignore
    /// assert!(5.12345f32.almost_eq(5.12, 0.1));
    /// assert!(0.00000001f32.almost_eq(0.0, 0.0000001));
    /// assert!(0.99999999f32.almost_eq(1.0, 0.0000001));
    /// ```
    fn almost_eq(&self, b: Self, accuracy: Self) -> bool;

    /// Variation of `almost_eq` that doesn't check for infinite or NaN values.
    fn almost_eq_fast(&self, b: Self, accuracy: Self) -> bool;
}

impl<T> AlmostEqExt for T where T: Float {
    fn almost_eq(&self, b: T, accuracy: T) -> bool {
        if self.is_infinite() || b.is_infinite() {
            *self == b
        } else if self.is_nan() && b.is_nan() {
            false
        } else {
            (*self - b).abs() < accuracy
        }
    }

    #[inline(always)]
    fn almost_eq_fast(&self, b: T, accuracy: T) -> bool {
        (*self - b).abs() < accuracy
    }
}

/// Linear interpolation for numeric types
///
/// ```
/// use combustion_common::num_utils::lerp;
///
/// assert_eq!(lerp(0.5f32, 0.0, 0.0, 1.0, 3.0), 1.5);
/// ```
pub fn lerp<T: Num + Copy>(x: T, x0: T, y0: T, x1: T, y1: T) -> T {
    y0 + (x - x0) * ((y1 - y0) / (x1 - x0))
}

/// Generic linear interpolation for any supported types.
///
/// This form can support non-numeric `T` types if they satisfy the clause conditions.
///
/// ```
/// use combustion_common::num_utils::lerp_generic as lerp;
///
/// assert_eq!(lerp(0.5f32, 0.0, 3.0), 1.5);
/// ```
pub fn lerp_generic<T, W: Num + Copy>(t: W, v0: T, v1: T) -> <<W as Mul<T>>::Output as Add>::Output
    where W: Mul<T>,
          T: Add<<W as Mul<T>>::Output>,
          <W as Mul<T>>::Output: Add<<W as Mul<T>>::Output> {
    (W::one() - t) * v0 + t * v1
}

/// Trait to add generic linear interpolation functionality to types directly.
///
/// ```
/// use combustion_common::num_utils::LerpExt;
///
/// // using generic form
/// assert_eq!(0.0f32.lerp_generic(0.5f32, 3.0), 1.5);
///
/// // using same-type form
/// assert_eq!(0.0f32.lerp(0.5, 3.0), 1.5);
/// ```
pub trait LerpExt: Sized {
    /// Linearly interpolate `self` with `other` based on the weight value `t`
    ///
    /// This is the generic form which can support non-numeric `Self` types if they satisfy the clause conditions.
    fn lerp_generic<W: Num + Copy>(self, t: W, other: Self) -> <<W as Mul<Self>>::Output as Add>::Output
        where W: Mul<Self>,
              Self: Add<<W as Mul<Self>>::Output>,
              <W as Mul<Self>>::Output: Add<<W as Mul<Self>>::Output> {
        (W::one() - t) * self + t * other
    }

    /// Linearly interpolate `self` with `other` based on the weight value `t`
    fn lerp(self, t: Self, other: Self) -> Self where Self: Num + Copy {
        (Self::one() - t) * self + t * other
    }
}

impl LerpExt for f32 {}

impl LerpExt for f64 {}