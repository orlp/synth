use std::error::Error;

pub fn log_if_error<T, E: Error>(msg: &str, result: Result<T, E>) -> Option<T> {
    match result {
        Ok(o) => Some(o),
        Err(e) => {
            eprintln!("{}: {}", msg, e);
            None
        }
    }
}

pub trait ExtraFloatOps {
    /// Equivalent to 
    fn mix(self, a: Self, b: Self) -> Self;
    fn mixexp(self, a: Self, b: Self) -> Self;
    fn db_to_gain(self) -> Self;
    fn gain_to_db(self) -> Self;
}

macro_rules! impl_extra_float_ops {
    ($T:ident) => {
        impl ExtraFloatOps for $T {
            #[inline(always)]
            fn mix(self, a: Self, b: Self) -> Self {
                a + self * (b - a)
            }

            #[inline(always)]
            fn mixexp(self, a: Self, b: Self) -> Self {
                a * (b / a).powf(self)
            }

            #[inline(always)]
            fn db_to_gain(self) -> Self {
                Self::from(10.0).powf(self * 0.05)
            }

            #[inline(always)]
            fn gain_to_db(self) -> Self {
                self.log10() * 20.0
            }
        }
    };
}

impl_extra_float_ops!(f32);
impl_extra_float_ops!(f64);
