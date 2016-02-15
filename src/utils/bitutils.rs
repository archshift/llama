#[inline]
pub fn sign_extend(data: u32, size: u32) -> i32 {
    assert!(size > 0 && size <= 32);
    ((data << (32 - size)) as i32) >> (32 - size)
}

#[macro_export]
macro_rules! extract_bits {
    ($val:expr, $low:expr => $hi:expr) => {{
        let max_bit = ::std::mem::size_of_val(&$val) * 8 - 1;
        $val << (max_bit - $hi) >> (max_bit - $hi + $low)
    }};
}

pub trait BitField<P, V> {
    fn get(parent: &P) -> V;
    fn set(parent: &mut P, val: V);
}

#[macro_export]
macro_rules! create_bitfield {
    ($name:ident: $ty:ty, { $($var_name:ident: $var_low:expr => $var_hi:expr),* }) => {
        #[allow(non_snake_case)]
        pub mod $name {
            $(
                #[allow(non_camel_case_types)]
                pub struct $var_name;

                impl ::utils::BitField<Type, $ty> for $var_name {
                    #[inline(always)]
                    #[allow(dead_code)]
                    fn get(parent: &Type) -> $ty {
                        extract_bits!(parent.val, $var_low => $var_hi)
                    }

                    #[inline(always)]
                    #[allow(dead_code)]
                    fn set(parent: &mut Type, val: $ty) {
                        parent.val ^= extract_bits!(parent.val, $var_low => $var_hi) << $var_low;
                        parent.val |= extract_bits!(val, $var_low => $var_hi) << $var_low;
                    }
                }
            )*

            #[derive(Clone)]
            pub struct Type {
                val: $ty
            }

            pub fn new(val: $ty) -> Type {
                Type {
                    val: val
                }
            }

            impl Type {
                #[inline(always)]
                #[allow(dead_code)]
                pub fn raw(&self) -> $ty {
                    self.val
                }

                #[inline(always)]
                #[allow(dead_code)]
                pub fn get<T: ::utils::BitField<Type, $ty>>(&self) -> $ty {
                    T::get(self)
                }

                #[inline(always)]
                #[allow(dead_code)]
                pub fn set<T: ::utils::BitField<Type, $ty>>(&mut self, val: $ty) {
                    T::set(self, val);
                }
            }

            impl ::std::fmt::Debug for Type {
                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    f.debug_struct(stringify!($name))
                        $(.field(stringify!($var_name), &self.get::<$var_name>()))*
                        .finish()
                }
            }
        }
    };
}
