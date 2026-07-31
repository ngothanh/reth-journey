use std::collections::VecDeque;
use PushOutcome::{AcceptedEvicting, Full, Rejected};

/// Extracted into `backpressure` crate W11.
pub enum BackpressureStrategy {
    DropOldest,
    DropNewest,
    Block,
}

pub struct BoundedBuffer<T> {
    strategy: BackpressureStrategy,
    buffer: VecDeque<T>,
    capacity: usize,
}

pub enum PushOutcome<T> {
    Accepted,
    AcceptedEvicting(T),
    Rejected(T),
    Full(T),
}

impl<T> BoundedBuffer<T> {
    pub fn new(strategy: BackpressureStrategy, capacity: usize) -> Self {
        assert!(capacity > 0);
        Self {
            strategy,
            capacity,
            buffer: VecDeque::with_capacity(capacity),
        }
    }

    pub fn try_push(&mut self, item: T) -> PushOutcome<T> {
        let len = self.buffer.len();
        if len < self.capacity {
            self.buffer.push_back(item);
            return PushOutcome::Accepted;
        }

        match self.strategy {
            BackpressureStrategy::DropOldest => {
                let t = self
                    .buffer
                    .pop_front()
                    .expect("drop oldest buffer shouldn't be empty");
                self.buffer.push_back(item);
                AcceptedEvicting(t)
            }
            BackpressureStrategy::DropNewest => Rejected(item),
            BackpressureStrategy::Block => Full(item),
        }
    }

    /// Dequeue the oldest item — the consumer side of the buffer.
    pub fn pop(&mut self) -> Option<T> {
        self.buffer.pop_front()
    }
}
