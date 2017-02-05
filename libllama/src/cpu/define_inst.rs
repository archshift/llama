#[macro_export]
macro_rules! __inst_gen_bf {
    (
        $itop:expr, [ {$fpart:expr}.$fwidth:expr $(; $part:tt.$width:expr)* ],
                    { $($spart:ident: $splow:expr => $sphi:expr),* }, $ty:ty
    ) => (
        // Move counter but otherwise pass it right through
        __inst_gen_bf!($itop - $fwidth, [ $($part.$width);* ],
                                        { $($spart: $splow => $sphi),* }, $ty);
    );

    (
        $itop:expr, [ $fpart:ident.$fwidth:expr $(; $part:tt.$width:expr)* ],
                    { $($spart:ident: $splow:expr => $sphi:expr),* }, $ty:ty
    ) => (
        __inst_gen_bf!($itop - $fwidth, [ $($part.$width);* ], {
                                            $($spart: $splow => $sphi,)*
                                            $fpart: ($itop - $fwidth) => ($itop - 1)
                                        }, $ty);
    );

    (
        $itop:expr, [ ], { $($spart:ident: $splow:expr => $sphi:expr),* }, $ty:ty
    ) => (
        bitfield!(InstrDesc: $ty, {
            $($spart: $splow => $sphi),*
        });
    )
}

#[macro_export]
macro_rules! __inst_gen_decode {
    (
        $itop:expr, [ {$fpart:expr}.$fwidth:expr $(; $part:tt.$width:expr)* ], $mask:expr, $test:expr, $ty:ty
    ) => (
        __inst_gen_decode!($itop - $fwidth, [ $($part.$width);* ],
                           $mask | (bits!(!0 as $ty,     0 => ($fwidth - 1)) << ($itop - $fwidth)),
                           $test | (bits!($fpart as $ty, 0 => ($fwidth - 1)) << ($itop - $fwidth)), $ty);
    );

    (
        $itop:expr, [ $fpart:ident.$fwidth:expr $(; $part:tt.$width:expr)* ], $mask:expr, $test:expr, $ty:ty
    ) => (
        // Move counter but otherwise pass it right through
        __inst_gen_decode!($itop - $fwidth, [ $($part.$width);* ], $mask, $test, $ty);
    );

    (
        $itop:expr, [ ], $mask:expr, $test:expr, $ty:ty
    ) => (
        #[inline(always)]
        pub fn try_decode(encoding: $ty) -> Option<InstrDesc> {
            assert!($itop == 0, format!("{} encoded bits do not add up!", module_path!()));
            if encoding & $mask == $test {
                Some(InstrDesc::new(encoding))
            } else {
                None
            }
        }
    )
}

#[macro_export]
macro_rules! define_inst {
    ($name:ident: $ty:ty, $($part:tt.$width:expr);* ) => (
        pub mod $name {
            __inst_gen_bf!(::std::mem::size_of::<$ty>()*8, [ $($part.$width);* ], { }, $ty);
            __inst_gen_decode!(::std::mem::size_of::<$ty>()*8, [ $($part.$width);* ], 0, 0, $ty);
        }
    )
}

#[macro_export]
macro_rules! define_insts {
    ($enumname:ident: $ty:ty, {
        $( $name:ident: [ $($part:tt.$width:expr);* ] ),*
    }) => (
        $( define_inst!($name: $ty, $($part.$width);*); )*

        #[derive(Debug)]
        #[allow(non_camel_case_types)]
        pub enum $enumname {
            $( $name($name::InstrDesc), )*
            Unknown
        }

        impl $enumname {
            pub fn decode(encoding: $ty) -> $enumname {
                $(
                    if let Some(desc) = $name::try_decode(encoding) {
                        return $enumname::$name(desc)
                    }
                )*

                $enumname::Unknown
            }
        }
    )
}

#[cfg(test)]
mod test {
    define_inst!(add_1: u16, {0b0001110}.7;immed_3.3;rn.3;rd.3);
    define_inst!(ldrh: u32, cond.4; {0b000}.3; p_bit.1; u_bit.1; i_bit.1; w_bit.1;
                            {0b1}.1; rn.4; rd.4; addr_mode.4; {0b1011}.4; addr_mode_.4);

    #[test]
    fn decode_add1() {
        assert!(add_1::try_decode(0b0001110000000000).is_some());
        assert!(add_1::try_decode(0b0011110000000000).is_none());
    }

    #[test]
    fn decode_ldrh() {
        let desc = ldrh::try_decode(0b11100000110101001001000010110100).unwrap();
        assert_eq!(bf!(desc.cond), 0b1110);
        assert_eq!(bf!(desc.rn), 4);
        assert_eq!(bf!(desc.rd), 9);
        assert!(ldrh::try_decode(0b11100000110101001001000011110100).is_none());
    }
}