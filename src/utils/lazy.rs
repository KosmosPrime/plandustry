//! [LazyLock] copy
use std::cell::UnsafeCell;
use std::mem::ManuallyDrop;
use std::panic::{RefUnwindSafe, UnwindSafe};

union Data<T, F> {
    value: ManuallyDrop<T>,
    f: ManuallyDrop<F>,
}

pub struct Lock<T, F = fn() -> T> {
    data: UnsafeCell<Data<T, F>>,
}

impl<T, F: FnOnce() -> T> Lock<T, F> {
    #[inline]
    pub const fn new(f: F) -> Lock<T, F> {
        Lock {
            data: UnsafeCell::new(Data {
                f: ManuallyDrop::new(f),
            }),
        }
    }

    #[inline]
    // SAFETY: CALL ONLY ONCE! NOT CHECKED
    pub unsafe fn load(this: &Lock<T, F>) {
        let data = &mut *this.data.get();
        let f = ManuallyDrop::take(&mut data.f);
        let value = f();
        data.value = ManuallyDrop::new(value);
    }
}

impl<T, F> Lock<T, F> {
    #[inline]
    // SAFETY: CALL [load] FIRST!
    pub unsafe fn get(&self) -> &T {
        &*(*self.data.get()).value
    }
}

unsafe impl<T: Sync + Send, F: Send> Sync for Lock<T, F> {}
impl<T: RefUnwindSafe + UnwindSafe, F: UnwindSafe> RefUnwindSafe for Lock<T, F> {}
impl<T: UnwindSafe, F: UnwindSafe> UnwindSafe for Lock<T, F> {}
