#[inline]
pub fn sign_extend(data: u32, size: u32) -> i32 {
    assert!(size > 0 && size <= 32);
    ((data << (32 - size)) as i32) >> (32 - size)
}

#[macro_export]
macro_rules! bits {
    ($val:expr, $low:expr => $hi:expr) => {{
        let max_bit = ::std::mem::size_of_val(&$val) * 8 - 1;
        $val << (max_bit - $hi) >> (max_bit - $hi + $low)
    }};
}

#[macro_export]
macro_rules! bit {
    ($val:expr, $bit:expr) => { bits!($val, $bit => $bit) };
}

#[macro_export]
macro_rules! bitfield {
    ($name:ident: $ty:ty, { $($var_name:ident: $var_low:expr => $var_hi:expr),* }) => {
        #[derive(Clone, Copy)]
        pub struct $name {
            val: $ty
        }

        impl $name {
            pub fn new(val: $ty) -> $name {
                $name {
                    val: val
                }
            }

            #[inline(always)]
            #[allow(dead_code)]
            pub fn raw(&self) -> $ty {
                self.val
            }

            #[inline(always)]
            #[allow(dead_code)]
            pub fn get(&self, pos: (usize, usize)) -> $ty {
                bits!(self.val, pos.0 => pos.1)
            }

            #[inline(always)]
            #[allow(dead_code)]
            pub fn set(&mut self, pos: (usize, usize), val: $ty) {
                self.val ^= bits!(self.val, pos.0 => pos.1) << pos.0;
                self.val |= bits!(val, 0 => pos.1 - pos.0) << pos.0;
            }

            $(
                #[inline(always)]
                pub fn $var_name() -> (usize, usize) {
                    ($var_low, $var_hi)
                }
            )*
        }

        impl ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                f.debug_struct(stringify!($name))
                    $(.field(stringify!($var_name), &self.get($name::$var_name())))*
                    .finish()
            }
        }
    };
}
