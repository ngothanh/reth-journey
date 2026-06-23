use std::cell::UnsafeCell;
use std::ops::Deref;
use std::process::abort;
use std::ptr::NonNull;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::sync::atomic::{fence, AtomicUsize};

struct ArcData<T> {
    data: UnsafeCell<Option<T>>,
    data_ref_count: AtomicUsize,
    allocate_ref_count: AtomicUsize,
}

pub struct Weak<T> {
    ptr: NonNull<ArcData<T>>,
}

pub struct Arc<T> {
    weak: Weak<T>,
}

unsafe impl<T: Send + Sync> Send for Weak<T> {}
unsafe impl<T: Send + Sync> Sync for Weak<T> {}

impl<T> Arc<T> {
    pub fn new(data: T) -> Self {
        let ptr = NonNull::from(Box::leak(Box::new(ArcData {
            data_ref_count: AtomicUsize::new(1),
            allocate_ref_count: AtomicUsize::new(1),
            data: UnsafeCell::new(Some(data)),
        })));
        Self { weak: Weak { ptr } }
    }

    pub fn get_mut(arc: &mut Self) -> Option<&mut T> {
        if arc.weak.data().allocate_ref_count.load(Relaxed) != 1 {
            return None;
        }

        fence(Acquire);
        //SAFETY: This is now the only reference
        let x = unsafe { arc.weak.ptr.as_mut() };
        Some(x.data.get_mut().as_mut().unwrap())
    }

    pub fn downgrade(arc: &Self) -> Weak<T> {
        arc.weak.clone()
    }
}

impl<T> Weak<T> {
    fn data(&self) -> &ArcData<T> {
        unsafe { self.ptr.as_ref() }
    }

    pub fn upgrade(&self) -> Option<Arc<T>> {
        let mut data_ref_count = self.data().data_ref_count.load(Relaxed);

        loop {
            if data_ref_count == 0 {
                return None;
            }
            assert!(data_ref_count <= usize::MAX / 2);
            if let Err(e) = self.data().data_ref_count.compare_exchange_weak(
                data_ref_count,
                data_ref_count + 1,
                Relaxed,
                Relaxed,
            ) {
                data_ref_count = e;
                continue;
            }

            return Some(Arc { weak: self.clone() });
        }
    }
}

impl<T> Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let x = self.weak.data().data.get();
        unsafe { (*x).as_ref().unwrap() }
    }
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        let weak = self.weak.clone();
        if weak.data().data_ref_count.fetch_add(1, Relaxed) > usize::MAX / 2 {
            abort();
        }
        Arc { weak }
    }
}

impl<T> Clone for Weak<T> {
    fn clone(&self) -> Self {
        if self.data().allocate_ref_count.fetch_add(1, Relaxed) > usize::MAX / 2 {
            abort()
        }

        Weak { ptr: self.ptr }
    }
}

impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        if self.weak.data().data_ref_count.fetch_sub(1, Release) == 1 {
            fence(Acquire);
            let ptr = self.weak.data().data.get();
            //SAFETY: Data ref == 1 so current thread is the only reference, so safe to mutate it
            unsafe {
                *ptr = None;
            };
        }
    }
}

impl<T> Drop for Weak<T> {
    fn drop(&mut self) {
        if self.data().allocate_ref_count.fetch_sub(1, Release) == 1 {
            fence(Acquire);
            drop(unsafe { Box::from_raw(self.ptr.as_ptr()) });
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::arc::Arc;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering::Relaxed;
    use std::thread;

    #[test]
    fn test_arc() {
        static NUM_DROPS: AtomicUsize = AtomicUsize::new(0);

        struct DetectDrop;

        impl Drop for DetectDrop {
            fn drop(&mut self) {
                NUM_DROPS.fetch_add(1, Relaxed);
            }
        }

        let x = Arc::new(("hello", DetectDrop)); //1 Arc
        let y = Arc::downgrade(&x); // 1 Weak
        let z = Arc::downgrade(&x); // 2 Weak

        let t = thread::spawn(move || {
            let y = y.upgrade().unwrap();  // 2 Arc - 1 Weak
            assert_eq!(y.0, "hello");
        });
        assert_eq!(x.0, "hello"); // 1 Arc - 1 Weak, y dropped. The Arc is x
        t.join().unwrap();

        assert_eq!(NUM_DROPS.load(Relaxed), 0); // x was not drop, then 0 NUM_DROP
        assert!(z.upgrade().is_some()); // tmp Arc is drop after assertion

        drop(x);
        assert_eq!(NUM_DROPS.load(Relaxed), 1); //DROP x, real data drop here
        assert!(z.upgrade().is_none());
    }
}
