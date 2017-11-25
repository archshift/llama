#[macro_export]
macro_rules! __inst_gen_bf {
    // We don't care about the value (`{val}.n`) parts. Reduces item list. Reduces $itop to next item.
    (
        $itop:expr, [ {$fpart:expr}.$fwidth:expr $(; $part:tt.$width:expr)* ],
                    { $($spart:ident: $splow:expr => $sphi:expr),* }, $ty:ty
    ) => (
        // Move counter but otherwise pass it right through
        __inst_gen_bf!($itop - $fwidth, [ $($part.$width);* ],
                                        { $($spart: $splow => $sphi),* }, $ty);
    );

    // Skip filler (`{}.n`) parts. Reduces item list. Reduces $itop to next item.
    (
        $itop:expr, [ {}.$fwidth:expr $(; $part:tt.$width:expr)* ],
                    { $($spart:ident: $splow:expr => $sphi:expr),* }, $ty:ty
    ) => (
        // Move counter but otherwise pass it right through
        __inst_gen_bf!($itop - $fwidth, [ $($part.$width);* ],
                                        { $($spart: $splow => $sphi),* }, $ty);
    );

    // From named (`foo.n`) parts, accumulate top and bottom bits into position list.
    // Reduces item list. Reduces $itop to next item.
    (
        $itop:expr, [ $fpart:tt.$fwidth:expr $(; $part:tt.$width:expr)* ],
                    { $($spart:ident: $splow:expr => $sphi:expr),* }, $ty:ty
    ) => (
        __inst_gen_bf!($itop - $fwidth, [ $($part.$width);* ], {
                                            $($spart: $splow => $sphi,)*
                                            $fpart: ($itop - $fwidth) => ($itop - 1)
                                        }, $ty);
    );

    // Once item list is empty, finally produce our end result: the `InstrDesc` bitfield.
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
    // ORs value (`{val}.n`) at position into $test, mask of value into $mask. Reduces item list.
    // Reduces $itop to next item.
    (
        $itop:expr, [ {$fpart:expr}.$fwidth:expr $(; $part:tt.$width:expr)* ], $mask:expr, $test:expr, $ty:ty
    ) => (
        __inst_gen_decode!($itop - $fwidth, [ $($part.$width);* ],
                           $mask | (bits!(!0 as $ty,     0 => ($fwidth - 1)) << ($itop - $fwidth)),
                           $test | (bits!($fpart as $ty, 0 => ($fwidth - 1)) << ($itop - $fwidth)), $ty);
    );

    // We don't care about the named (`foo.n`) parts. Reduces item list. Reduces $itop to next item.
    (
        $itop:expr, [ $fpart:tt.$fwidth:expr $(; $part:tt.$width:expr)* ], $mask:expr, $test:expr, $ty:ty
    ) => (
        // Move counter but otherwise pass it right through
        __inst_gen_decode!($itop - $fwidth, [ $($part.$width);* ], $mask, $test, $ty);
    );

    // Skip filler (`{}.n`) parts. Reduces item list. Reduces $itop to next item.
    (
        $itop:expr, [ {}.$fwidth:expr $(; $part:tt.$width:expr)* ], $mask:expr, $test:expr, $ty:ty
    ) => (
        // Move counter but otherwise pass it right through
        __inst_gen_decode!($itop - $fwidth, [ $($part.$width);* ], $mask, $test, $ty);
    );

    // Once item list is empty, finally produce our end result: the `decodable` function.
    (
        $itop:expr, [ ], $mask:expr, $test:expr, $ty:ty
    ) => (
        #[inline(always)]
        pub fn decodable(encoding: $ty) -> bool {
            assert!($itop == 0, format!("{} encoded bits do not add up!", module_path!()));
            encoding & $mask == $test
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

// Usage:
// define_insts!(Foo: ty {
//     with [ constraints ]
//       or [ constraints ]...
//     {
//         bar: [ constraints ],...
//     }
//     ...
// });
#[macro_export]
macro_rules! define_insts {
    ($enumname:ident: $ty:ty, {
        $(
            with $([ $($wpart:tt.$wwidth:expr);* ])or*
            {
                $( $name:ident: [ $($part:tt.$width:expr);* ] ),*
            }
        )*
    }) => (
        $($( define_inst!($name: $ty, $($part.$width);*); )*)*

        #[derive(Clone, Copy, Debug)]
        #[allow(non_camel_case_types)]
        pub enum $enumname {
            $($( $name($name::InstrDesc), )*)*
            Unknown
        }

        impl $enumname {
            pub fn decode(encoding: $ty) -> $enumname {
                $(
                    if $({ __inst_gen_decode!(::std::mem::size_of::<$ty>()*8, [ $($wpart.$wwidth);* ], 0, 0, $ty);
                         decodable(encoding) })||* {
                    $(
                        if $name::decodable(encoding) {
                            return $enumname::$name($name::InstrDesc::new(encoding))
                        }
                    )*
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
        assert!(add_1::decodable(0b0001110000000000));
        assert!(!add_1::decodable(0b0011110000000000));
    }

    #[test]
    fn decode_ldrh() {
        let ldrh_bits = 0b11100000110101001001000010110100;
        assert!(ldrh::decodable(ldrh_bits));

        let desc = ldrh::InstrDesc::new(0b11100000110101001001000010110100);
        assert_eq!(bf!(desc.cond), 0b1110);
        assert_eq!(bf!(desc.rn), 4);
        assert_eq!(bf!(desc.rd), 9);
        assert!(!ldrh::decodable(0b11100000110101001001000011110100));
    }
}