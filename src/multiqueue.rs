use anyhow::*;
use std::option::*;
use std::sync::*;

struct MQItem<T> {
    data: T,
    next: Arc<Option<Box<MQItem<T>>>>,
}
impl<T> Iterator for MultiQueue<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
pub struct MultiQueue<T> {
    head: Arc<Option<Box<MQItem<T>>>>,
}
impl<T> MQItem<T> {
    fn push(&self, item: T) {
        if let Some(h) = self.next {
            h.next.push(item)
        } else {
            self.data = item;
        }
    }
}
impl<T> MultiQueue<T> {
    pub fn new() -> MultiQueue<T> {
        MultiQueue {
            head: Arc::new(None),
        }
    }
    pub fn pull(&self) -> Option<T> {
        let h = self.head.as_ref();
        match h {
            None => None,
            Some(i) => {
                self.head = i.next;
                Some(i.data)
            }
        }
    }
    pub fn push(&self, item: T) -> anyhow::Result<()> {
        if let Some(h) = self.head.unwrap() {
            h.push(item)
        } else {
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let q: MultiQueue<&str> = MultiQueue::new(10);
        q.push("one");
        let mut i = q.pull_until(&|i| i == "two");
        q.push("two");
        q.push("three");
        assert_eq!("two", i.next().unwrap());
        assert_eq!(std::option::Option::None, i.next());
    }
}
