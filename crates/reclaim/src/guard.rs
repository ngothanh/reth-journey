use crate::root::Root;
use crate::sync::Ordering;
pub trait Guard {
    fn try_protect<T>(&mut self, addr: *mut T, root: &Root<T>) -> Result<*mut T, *mut T>;

    fn protect<T>(&mut self, root: &Root<T>) -> *mut T {
        let mut holding_addr = root.ptr().load(Ordering::Acquire);
        loop {
            match self.try_protect(holding_addr, root) {
                Ok(p) => {
                    return p;
                }
                Err(new_addr) => {
                    holding_addr = new_addr;
                }
            }
        }
    }
}
