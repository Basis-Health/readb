pub type Key = (u64, usize);
type Value = Vec<u8>;

pub trait Cache {
    fn new(size: usize) -> Self;
    fn new_default() -> Self;

    fn get(&mut self, key: &Key) -> Option<Value>;
    fn put(&mut self, key: Key, value: Value);

    fn invalidate(&mut self);
}

pub(crate) mod lfu;
