use std::sync::atomic::{self, Ordering};
use std::sync::Arc;
use std::cell::UnsafeCell;
use std::marker::PhantomData;

pub unsafe trait AtomicDat: Default {
    type DAT: Copy;
    fn atomic_load(&self, ord: Ordering) -> Self::DAT;
    fn atomic_store(&self, val: Self::DAT, ord: Ordering);
}

pub struct Broadcaster<T: AtomicDat> {
    val: Arc<T>,
    disallow_sync: PhantomData<UnsafeCell<()>>,
}

impl<T: AtomicDat> Broadcaster<T> {
    pub fn new() -> (Self, Audience<T>) {
        let val: Arc<T> = Arc::new(Default::default());
        let out = Self {
            val: val.clone(),
            disallow_sync: PhantomData
        };
        (out, Audience { val })
    }

    pub fn update(&mut self, val: T::DAT) {
        self.val.atomic_store(val, Ordering::Relaxed)
    }
    pub fn explicit_update(&mut self, val: T::DAT, ord: Ordering) {
        self.val.atomic_store(val, ord)
    }
}

#[derive(Clone)]
pub struct Audience<T: AtomicDat> {
    val: Arc<T>
}

impl<T: AtomicDat> Audience<T> {
    pub fn val(&self) -> T::DAT {
        self.val.atomic_load(Ordering::Relaxed)
    }
    pub fn explicit_val(&self, ord: Ordering) -> T::DAT {
        self.val.atomic_load(ord)
    }
}




macro_rules! atomic_impl {
    ($($which:ty = $dat:ty);* $(;)*) => { $(
        unsafe impl AtomicDat for $which {
            type DAT = $dat;
            fn atomic_load(&self, ord: Ordering) -> Self::DAT {
                self.load(ord)
            }
            fn atomic_store(&self, val: Self::DAT, ord: Ordering) {
                self.store(val, ord)
            }
        }
    )* }
}

atomic_impl! {
    atomic::AtomicIsize = isize;
    atomic::AtomicUsize = usize;
    atomic::AtomicBool = bool;
}
