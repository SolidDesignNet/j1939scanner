use std::fmt::Display;
use std::option::*;
use std::sync::*;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use std::time::Instant;

/// Linked list data
struct MqItem<T> {
    data: T,
    next: MqNode<T>,
}
/// Linked list nodes
type MqNode<T> = Arc<RwLock<Option<MqItem<T>>>>;

#[derive(Clone)]
pub struct MultiQueue<T> {
    // shared head that always points to the empty Arc<RwLock>
    // Yes, this seems like overkill, but we need to clone multiqueues to easily use them in threads, so this make cloning work easily.
    head: Arc<RwLock<MqNode<T>>>,
}

/// Iterator
struct MqIter<T> {
    head: MqNode<T>,
    until: Instant,
}

impl<T> Iterator for MqIter<T>
where
    T: Clone + Sync + Send,
{
    type Item = T;
    fn next(&mut self) -> std::option::Option<<Self as std::iter::Iterator>::Item> {
        let mut o = None;
        while o.is_none() && Instant::now() < self.until {
            thread::yield_now();
            o = self
                .head
                .read()
                .unwrap()
                .as_ref()
                .map(|i| (i.data.clone(), i.next.clone()));
        }
        o.map(|clones| {
            self.head = clones.1;
            clones.0
        })
    }
}

#[allow(dead_code)]
impl<T> MultiQueue<T>
where
    T: Clone + Sync + Send,
{
    pub fn new() -> MultiQueue<T> {
        MultiQueue {
            head: Arc::new(RwLock::new(Arc::new(RwLock::new(None)))),
        }
    }
    pub fn iter_for(&self, duration: Duration) -> impl Iterator<Item = T> {
        MqIter {
            head: self.head.read().unwrap().clone(),
            until: Instant::now() + duration,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = T> {
        self.iter_for(Duration::from_secs(0))
    }

    pub fn push(&mut self, item: T) {
        let empty = Arc::new(RwLock::new(None));
        let mut head = self.head.write().unwrap();
        // add the new item.
        *head.write().unwrap() = Some(MqItem {
            data: item,
            next: empty.clone(),
        });
        // update head to point to the new empty item.
        *head = empty;
    }
}

#[allow(dead_code)]
impl<T> MultiQueue<T>
where
    T: Clone + Sync + Send + Display + 'static,
{
    /// Lazy man's debugging
    pub fn log(&self) -> JoinHandle<()> {
        let mut iter = self.iter();
        std::thread::spawn(move || loop {
            std::thread::yield_now();
            if let Some(p) = iter.next() {
                println!("{}", p)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let mut q: MultiQueue<&str> = MultiQueue::new();
        q.push("one");
        let mut i = q.iter();
        q.push("two");
        q.push("three");
        assert_eq!("two", i.next().unwrap());
        assert_eq!("three", i.next().unwrap());
        assert_eq!(std::option::Option::None, i.next());
    }
}
