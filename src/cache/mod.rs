pub trait Cache {
    fn new(size: usize) -> Self;
    fn new_default() -> Self;

    fn get(&mut self, key: &usize) -> Option<String>;
    fn put(&mut self, key: usize, value: String);
}

pub(crate) mod lfu;
