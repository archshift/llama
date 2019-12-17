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
        if self.len() >= self.max_len {
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

    pub fn free_space(&self) -> usize {
        self.max_len - self.len()
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn fifo_bounds() {
        let mut fifo = Fifo::<u32>::new(16);
        assert!(fifo.empty());
        assert!(fifo.pop().is_none());

        for i in 0..16 {
            let ok = fifo.push(i);
            assert!(ok);
        }
        assert!(fifo.full());
        assert!(!fifo.push(16));

        for i in 0..16 {
            let out = fifo.pop().unwrap();
            assert_eq!(out, i);
        }

        assert!(fifo.empty());
        assert!(fifo.pop().is_none());
    }

    #[test]
    fn fifo_drain() {
        let mut fifo = Fifo::<u32>::new(16);
        for i in 0..16 {
            fifo.push(i);
        }

        let mut drain_buf = [0; 16];
        let amount = fifo.drain(&mut drain_buf[..4]);
        assert_eq!(amount, 4);
        assert_eq!(&drain_buf[..4], &(0..4).collect::<Vec<_>>()[..]);
        assert_eq!(fifo.len(), 12);

        let amount = fifo.drain(&mut drain_buf);
        assert_eq!(amount, 12);
        assert_eq!(&drain_buf[..12], &(4..16).collect::<Vec<_>>()[..]);
        assert!(fifo.empty());
    }

    #[test]
    fn fifo_extend() {
        let mut fifo = Fifo::<u32>::new(16);
        let in_buf: Vec<_> = (0..16).collect();

        let amount = fifo.clone_extend(&in_buf[..4]);
        assert_eq!(amount, 4);
        assert_eq!(fifo.len(), 4);

        let amount = fifo.clone_extend(&in_buf);
        assert_eq!(amount, 12);
        assert!(fifo.full());

        let mut drain_buf = [0; 16];
        let amount = fifo.drain(&mut drain_buf[..4]);
        assert_eq!(amount, 4);
        assert_eq!(&drain_buf[..4], &in_buf[..4]);

        let amount = fifo.drain(&mut drain_buf);
        assert_eq!(amount, 12);
        assert_eq!(&drain_buf[..12], &in_buf[..12]);
    }
}