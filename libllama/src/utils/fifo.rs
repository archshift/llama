use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct Fifo<T> {
    inner: VecDeque<T>,
    max_len: usize,
}

impl<T> Fifo<T> {
    pub fn new(max_len: usize) -> Self {
        let mut inner = VecDeque::new();
        inner.reserve_exact(max_len);
        Self {
            inner,
            max_len
        }
    }


    pub fn push(&mut self, item: T) -> bool {
        if self.len() > self.max_len {
            return false
        }

        self.inner.push_back(item);
        true
    }

    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop_front()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn drain(&mut self, dst: &mut [T]) -> usize {
        let drain_amount = dst.len().min(self.len());
        for i in 0..drain_amount {
            dst[i] = self.pop().unwrap();
        }
        drain_amount
    }

    pub fn full(&self) -> bool {
        self.len() == self.max_len
    }

    pub fn empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T: Clone> Fifo<T> {
    pub fn clone_extend(&mut self, items: &[T]) -> usize {
        let space_left = self.max_len - self.len();
        let copy_amount = items.len().min(space_left);

        self.inner.extend(items[..copy_amount].iter().cloned());

        copy_amount
    }
}
