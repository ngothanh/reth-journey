#[cfg(not(loom))]
mod tests {
    use concurrent::RwLock;

    struct Inner {
        a: i32,
        b: i32,
        c: i32,
        d: i32,
    }

    #[test]
    fn read_write_exclusively() {
        let inner = Inner {
            a: 1,
            b: 2,
            c: 3,
            d: 4,
        };

        let lock = RwLock::new(inner);
        assert_eq!(lock.read().a, 1);
        lock.write().a = 2;
        assert_eq!(lock.read().a, 2);
    }

    #[test]
    fn read_sees_initial_value() {
        let lock = RwLock::new(42);

        assert_eq!(*lock.read(), 42);
    }

    #[test]
    fn write_mutates_then_read_observes() {
        let lock = RwLock::new(42);
        *lock.write() = 45;

        assert_eq!(*lock.read(), 45);
    }

    #[test]
    fn two_read_guards_coexist() {
        let lock = RwLock::new(43);
        let r1 = lock.read();
        let r2 = lock.read();

        assert_eq!(*r1, 43);
        assert_eq!(*r2, 43);
    }

    #[test]
    fn write_then_write_after_drop() {
        let lock = RwLock::new(42);
        {
            let mut w1 = lock.write();
            *w1 = 45;
        }

        {
            let mut w2 = lock.write();
            *w2 = 43;
        }

        assert_eq!(*lock.read(), 43);
    }
}
