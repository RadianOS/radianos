pub struct StaticVec<T, const N: usize> {
    inner: [T; N],
    size: usize,
}
impl<T, const N: usize> StaticVec<T, N> {
    pub fn new() -> Self where T: Default {
        Self{
            inner: core::array::from_fn(|_| T::default()),
            size: 0
        }
    }

    pub fn push(&mut self, data: T) {
        self.inner[self.size] = data;
        self.size += 1;
    }

    pub fn pop(&mut self) -> Option<T> where T: Default {
        if self.size == 0 {
            None
        } else {
            let opt = core::mem::take(&mut self.inner[self.size]);
            self.size -= 1;
            Some(opt)
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.size
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        self.inner.get(index)
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.inner.get_mut(index)
    }
}
