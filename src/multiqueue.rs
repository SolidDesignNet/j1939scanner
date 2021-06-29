use anyhow::*;
use std::option::*;
use std::sync::*;

#[derive(Debug)]
struct MQData<T> {
    end: u64,
    vec: Vec<T>,
}

#[derive(Debug)]
pub struct MultiQueue<T> {
    // count of items.  Not wrapped to vec.
    data: Arc<RwLock<MQData<T>>>,
    start: u64,
    size: u64,
}
pub struct MQIterator<T> {
    queue: MultiQueue<T>,
}
impl<T> Iterator for MQIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
impl<T> Clone for MultiQueue<T> {
    fn clone(&self) -> Self {
        todo!()
    }
}
impl<T> MultiQueue<T> {
    pub fn new(capacity: usize) -> MultiQueue<T> {
        MultiQueue {
            data: Arc::new(RwLock::new(MQData {
                end: 0,
                vec: Vec::with_capacity(capacity),
            })),
            start: 0,
            size: capacity as u64,
        }
    }
    fn get(&self, index: u64) -> Option<&T> {
        if index >= self.start {
            if let Ok(t) = self.data.read() {
                if index < t.end {
                    let i = index - self.start;
                    return t.vec.get((i % self.size) as usize);
                }
            }
        }
        Option::None
    }
    pub fn push(&self, item: T) -> anyhow::Result<()> {
        let lock = self.data.write();
        match lock {
            Ok(mut t) => {
                t.vec.push(item);
                t.end += 1;
                Ok(())
            }
            Err(e) => Err(anyhow!("")),
        }
    }
    pub fn pull_until(&self, untilFn: &'static dyn Fn(T) -> bool) -> MQIterator<T> {
        //MQIterator { queue: *self }
        todo!()
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
