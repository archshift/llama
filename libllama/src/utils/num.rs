macro_rules! wrapping_sum {
    ($a:expr, $b:expr $(, $rest:expr)*) => (
        $a.wrapping_add($b)
            $(.wrapping_add($rest))*
    )
}

macro_rules! checked_sum {
    ($a:expr, $b:expr $(, $rest:expr)*) => (
        $a.checked_add($b)
            $(.and_then(|x| x.checked_add($rest)))*
    )
}

macro_rules! wrapping_diff {
    ($a:expr, $b:expr $(, $rest:expr)*) => (
        $a.wrapping_sub($b)
            $(.wrapping_sub($rest))*
    )
}

macro_rules! checked_diff {
    ($a:expr, $b:expr $(, $rest:expr)*) => (
        $a.checked_sub($b)
            $(.and_then(|x| x.checked_sub($rest)))*
    )
}
