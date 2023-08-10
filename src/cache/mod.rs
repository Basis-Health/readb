type Key = (usize, usize);
type Value = Vec<u8>;

pub trait Cache {
    fn new(size: usize) -> Self;
    fn new_default() -> Self;

    fn get(&mut self, key: &Key) -> Option<Value>;
    fn put(&mut self, key: Key, value: Value);
}

pub(crate) mod lfu;
