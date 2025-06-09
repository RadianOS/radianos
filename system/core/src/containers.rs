use core::ops::{Index, IndexMut};

#[derive(Debug, Clone)]
pub struct StaticVec<T, const N: usize> {
    inner: [T; N],
    size: usize,
}
impl<T: Default, const N: usize> Default for StaticVec<T, N> {
    fn default() -> Self {
        Self {
            inner: core::array::from_fn(|_| T::default()),
            size: 0,
        }
    }
}
impl<T, const N: usize> StaticVec<T, N> {
    pub fn new() -> Self
    where
        T: Default,
    {
        Self::default()
    }

    pub fn push(&mut self, data: T) {
        self.inner[self.size] = data;
        self.size += 1;
    }
    
    /// Pushes and drops the first element
    pub fn push_fifo(&mut self, data: T) where T: Default {
        if self.len() >= self.max_len() {
            let t = self.pop().unwrap();
            self[0] = t;
        }
        self.push(data);
    }

    pub fn pop(&mut self) -> Option<T>
    where
        T: Default,
    {
        if self.size == 0 {
            None
        } else {
            let opt = core::mem::take(&mut self.inner[self.size - 1]);
            self.size -= 1;
            Some(opt)
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.size
    }

    #[inline]
    pub fn max_len(&self) -> usize {
        N
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.size == 0
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

impl<T, const N: usize> Index<usize> for StaticVec<T, N> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}
impl<T, const N: usize> IndexMut<usize> for StaticVec<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
    }
}

#[derive(Debug, Clone)]
pub struct StaticString<const N: usize> {
    inner: [u8; N],
    size: usize,
}
impl<const N: usize> StaticString<N> {
    pub fn new() -> Self {
        Self{
            inner: [0; N],
            size: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.size
    }
    pub fn max_len(&self) -> usize {
        N
    }
    pub fn bytes<'a>(&'a self) -> &'a [u8; N] {
        &self.inner
    }
    pub fn bytes_mut<'a>(&'a mut self) -> &'a mut [u8; N] {
        &mut self.inner
    }
    pub fn from_str(s: &str) -> Self {
        let mut inner = [0u8; N];
        for (i, b) in s.bytes().enumerate() {
            inner[i] = b;
        }
        Self{
            inner,
            size: s.len(),
        }
    }
    pub fn as_str(&self) -> &str {
        let len = self
            .inner
            .iter()
            .enumerate()
            .find(|&(_, c)| *c == 0)
            .map(|(i, _)| i)
            .unwrap_or(self.inner.len());
        unsafe { core::str::from_raw_parts(self.inner.as_ptr(), len) }
    }
}
impl<const N: usize> Default for StaticString<N> {
    fn default() -> Self {
        Self::new()
    }
}
