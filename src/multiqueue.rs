use anyhow::*;
use std::option::*;
use std::sync::*;

#[derive(Clone, Debug)]
pub struct MultiQueue<T> {
    data: Arc<RwLock<Vec<T>>>,
    start: u64,
    end: Arc<RwLock<u64>>,
}
pub struct MQIterator<T> {
    queue: MultiQueue<T>,
    untilFn: &'static dyn Fn(T) -> bool,
}
impl<T> Iterator for MQIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        // FIXME
        Option::None
    }
}

impl<T> MultiQueue<T> {
    pub fn new() -> MultiQueue<T> {
        MultiQueue {
            data: Arc::new(RwLock::new(Vec::new())),
            start: 0,
            end: Arc::new(RwLock::new(0)),
        }
    }
    fn get(&self, index: u64) -> Option<&T> {
        if index >= self.start && index < self.end.borrow() {
            self.data.get((index - self.start) as usize)
        } else {
            Option::None
        }
    }
    pub fn push(&self, item: T) -> Result<()> {
        Ok(())
    }
    pub fn pull(&self, untilFn: &'static dyn Fn(T) -> bool) -> MQIterator<T> {
        MQIterator {
            queue: *self,
            untilFn,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let q: MultiQueue<&str> = MultiQueue::new();
        q.push("one");
        let i = q.pull(&|i| i == "two");
        q.push("two");
        q.push("three");
        assert_eq!("two", i.next().unwrap());
        assert_eq!(std::option::Option::None, i.next());
    }
}
