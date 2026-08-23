#![allow(
    non_snake_case,
    non_upper_case_globals,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::missing_panics_doc
)]

/// XNA single-precision math helpers.
pub struct MathHelper;

impl MathHelper {
    pub const E: f32 = core::f32::consts::E;
    pub const Log10E: f32 = core::f32::consts::LOG10_E;
    pub const Log2E: f32 = core::f32::consts::LOG2_E;
    pub const Pi: f32 = core::f32::consts::PI;
    pub const PiOver2: f32 = core::f32::consts::FRAC_PI_2;
    pub const PiOver4: f32 = core::f32::consts::FRAC_PI_4;
    pub const TwoPi: f32 = core::f32::consts::TAU;

    #[must_use]
    pub fn Clamp(value: f32, min: f32, max: f32) -> f32 {
        if value > max {
            max
        } else if value < min {
            min
        } else {
            value
        }
    }

    #[must_use]
    pub fn Distance(value1: f32, value2: f32) -> f32 {
        (value1 - value2).abs()
    }

    #[must_use]
    pub fn Min(value1: f32, value2: f32) -> f32 {
        if value1.is_nan() || value1 < value2 {
            value1
        } else {
            value2
        }
    }

    #[must_use]
    pub fn Max(value1: f32, value2: f32) -> f32 {
        if value1.is_nan() || value1 > value2 {
            value1
        } else {
            value2
        }
    }

    #[must_use]
    pub fn Lerp(value1: f32, value2: f32, amount: f32) -> f32 {
        value1 + (value2 - value1) * amount
    }

    #[must_use]
    pub fn Barycentric(value1: f32, value2: f32, value3: f32, amount1: f32, amount2: f32) -> f32 {
        value1 + amount1 * (value2 - value1) + amount2 * (value3 - value1)
    }

    #[must_use]
    pub fn SmoothStep(value1: f32, value2: f32, amount: f32) -> f32 {
        let amount = Self::Clamp(amount, 0.0, 1.0);
        Self::Lerp(value1, value2, amount * amount * (3.0 - 2.0 * amount))
    }

    #[must_use]
    pub fn CatmullRom(value1: f32, value2: f32, value3: f32, value4: f32, amount: f32) -> f32 {
        let squared = amount * amount;
        let cubed = amount * squared;
        0.5 * (2.0 * value2
            + (-value1 + value3) * amount
            + (2.0 * value1 - 5.0 * value2 + 4.0 * value3 - value4) * squared
            + (-value1 + 3.0 * value2 - 3.0 * value3 + value4) * cubed)
    }

    #[must_use]
    pub fn Hermite(value1: f32, tangent1: f32, value2: f32, tangent2: f32, amount: f32) -> f32 {
        let squared = amount * amount;
        let cubed = amount * squared;
        let first = 2.0 * cubed - 3.0 * squared + 1.0;
        let second = -2.0 * cubed + 3.0 * squared;
        let third = cubed - 2.0 * squared + amount;
        let fourth = cubed - squared;
        value1 * first + value2 * second + tangent1 * third + tangent2 * fourth
    }

    #[must_use]
    pub fn ToDegrees(radians: f32) -> f32 {
        radians * 57.295_78
    }

    #[must_use]
    pub fn ToRadians(degrees: f32) -> f32 {
        degrees * 0.017_453_292
    }

    #[must_use]
    pub fn WrapAngle(angle: f32) -> f32 {
        let mut angle = Self::ieee_remainder(angle, Self::TwoPi);
        if angle <= -Self::Pi {
            angle += Self::TwoPi;
        } else if angle > Self::Pi {
            angle -= Self::TwoPi;
        }
        angle
    }

    fn ieee_remainder(value: f32, divisor: f32) -> f32 {
        if !value.is_finite() || divisor == 0.0 || divisor.is_nan() {
            return f32::NAN;
        }
        let value64 = f64::from(value);
        let divisor64 = f64::from(divisor);
        let quotient = value64 / divisor64;
        let floor = quotient.floor();
        let fraction = quotient - floor;
        let nearest = if fraction < 0.5
            || (fraction == 0.5
                && floor >= i64::MIN as f64
                && floor <= i64::MAX as f64
                && floor as i64 % 2 == 0)
        {
            floor
        } else {
            floor + 1.0
        };
        let result = (value64 - divisor64 * nearest) as f32;
        if result == 0.0 {
            result.copysign(value)
        } else {
            result
        }
    }
}
