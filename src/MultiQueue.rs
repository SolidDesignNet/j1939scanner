pub struct MultiQueue {
    data: Vec<Arc<T>>  = Vec::new(32);
    start: uint32;
    end: uint32;
}

impl<T> MultiQueue<T> {
    pub fn push(&self, T item) -> Self {
        if (start+1==end){
            // resize
            let new_data = Vec<Arc<T>>::new(data.capacity() * 2);
            // copy from start
            // copy from data[0]
            data = new_data;
            data.push(item)
        }else        if (){
            // gap before start
        }else {
            // gap after start
        }
    }
    pub fn pull(&self,until:Fn)->Iter<T>{}
}

#[cfg(test)]
mod tests {
    #[test]
    fn simple(){
        lec c = Rc();
        q = MultiQueue<@str>;
        q.push("one");
        i = q.pull(|i|i=="two");
        q.push("two")
         .push("three");
        assert_eq!("two",i.next().unwrap());
        assert_eq!(Option.NONE,i.next());
    }
}