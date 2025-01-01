pub struct ThreadSafePtr<T>(pub *mut T);

unsafe impl<T> Send for ThreadSafePtr<T> {}
unsafe impl<T> Sync for ThreadSafePtr<T> {}

impl<T> ThreadSafePtr<T> {
    pub fn deref(&self) -> &'static T {
        unsafe { &*self.0 }
    }

    pub fn deref_mut(self) -> &'static mut T {
        unsafe { &mut *self.0 }
    }
}
