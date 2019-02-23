pub mod graphics;
pub mod skia;
mod skia_euclid;

#[macro_use]
extern crate bitflags;

mod prelude {
    use std::intrinsics::transmute;
    #[cfg(test)]
    use rust_skia::{SkSurface, SkData, SkColorSpace};
    use rust_skia::{
        SkNVRefCnt,
        SkRefCnt,
        SkRefCntBase,
    };
    // export all traits for the use of points / vectors, sizes,
    // etc. into the prelude.
    pub use crate::skia_euclid::{
        SkiaPoint,
        SkiaPointFloat,
        SkiaSize,
        SkiaSizeFloat,
        SkiaRect,
        SkiaRectFloat
    };
    use std::ptr;

    pub trait ToOption {
        type Target;

        fn to_option(self) -> Option<Self::Target>;
    }

    impl<T> ToOption for *mut T {
        type Target = *mut T;

        fn to_option(self) -> Option<Self::Target> {
            if self.is_null()
            { None } else { Some(self) }
        }
    }

    pub trait RefCount {
        fn ref_cnt(&self) -> i32;
    }

    impl RefCount for SkRefCntBase {
        // the problem here is that the binding generator represents std::atomic as an u8 (we
        // are lucky that the C alignment rules make space for an i32), so to get the ref
        // counter, we need to get the u8 pointer to fRefCnt and interpret it as an i32 pointer.
        fn ref_cnt(&self) -> i32 {
            let ptr: *const i32 = unsafe { transmute(&self.fRefCnt) };
            unsafe { *ptr }
        }
    }
    impl RefCount for SkRefCnt {
        fn ref_cnt(&self) -> i32 {
            self._base.ref_cnt()
        }
    }

    impl RefCount for SkNVRefCnt {
        fn ref_cnt(&self) -> i32 {
            let ptr: *const i32 = unsafe { transmute(&self.fRefCnt) };
            unsafe { *ptr }
        }
    }

    #[cfg(test)]
    impl RefCount for SkData {
        fn ref_cnt(&self) -> i32 {
            self._base.ref_cnt()
        }
    }

    #[cfg(test)]
    impl RefCount for SkSurface {
        fn ref_cnt(&self) -> i32 {
            self._base.ref_cnt()
        }
    }

    #[cfg(test)]
    impl RefCount for SkColorSpace {
        fn ref_cnt(&self) -> i32 {
            self._base.ref_cnt()
        }
    }

    /// Supporting trait for the derive macro RCCloneDrop.
    pub trait RefCounted : Sized {
        fn _ref(&self);
        fn _unref(&self);
    }

    /// Indicates that the type has a native representation and
    /// can convert to and from it. This is for cases in which we
    /// can't use the From / Into traits, because we pull in the
    /// rust type from another crate.
    pub trait NativeRepresentation<Native> {
        fn to_native(self) -> Native;
        fn from_native(native: Native) -> Self;
    }

    /// Trait that enables access to a native representation by reference.
    pub trait NativeAccess<N> {
        fn native(&self) -> &N;
        fn native_mut(&mut self) -> &mut N;
    }

    /// Every handle's native type needs to imlement this trait.
    pub trait NativeDrop {
        fn drop(&mut self);
    }

    /// A representation type for a native type that needs a destructor to be called.
    pub struct Handle<N: NativeDrop>(N);

    impl<N: NativeDrop> Handle<N> {
        #[inline]
        pub fn from_native(n: N) -> Handle<N> {
            Handle(n)
        }
    }

    impl<N: NativeDrop> NativeAccess<N> for Handle<N> {
        fn native(&self) -> &N {
            &self.0
        }

        fn native_mut(&mut self) -> &mut N {
            &mut self.0
        }
    }

    /// A trait that supports retrieving a pointer from an Option<Handle<Native>>
    pub trait NativePointer<N> {
        fn native_ptr(&self) -> *const N;
    }

    impl<N: NativeDrop> NativePointer<N> for Option<Handle<N>> {
        #[inline]
        fn native_ptr(&self) -> *const N {
            match self {
                Some(handle) => handle.native(),
                None => ptr::null()
            }
        }
    }

    /// A representation type represented by a refcounted pointer to the native type.
    pub struct RCHandle<Native: RefCounted>(*mut Native);

    impl<N: RefCounted> RCHandle<N> {
        /// Increases the reference counter of the native type
        /// and returns a mutable reference.
        #[inline]
        pub fn shared_native(&self) -> &mut N {
            (unsafe { &*self.0 })._ref();
            unsafe { &mut *self.0 }
        }

        /// Constructs from a pointer. Returns None if the pointer is None.
        /// Does not increase the reference count.
        #[inline]
        pub fn from_ptr(ptr: *mut N) -> Option<Self> {
            if !ptr.is_null() {
                Some(RCHandle(ptr))
            } else {
                None
            }
        }
    }

    impl<N: RefCounted> NativeAccess<N> for RCHandle<N> {
        /// Returns a reference to the native representation.
        #[inline]
        fn native(&self) -> &N {
            unsafe { &*self.0 }
        }

        /// Returns a mutable reference to the native representation.
        #[inline]
        fn native_mut(&mut self) -> &mut N {
            unsafe { &mut *self.0 }
        }
    }

    impl<N: RefCounted> Clone for RCHandle<N> {
        #[inline]
        fn clone(&self) -> Self {
            RCHandle(self.shared_native())
        }
    }

    impl <N: RefCounted> Drop for RCHandle<N> {
        #[inline]
        fn drop(&mut self) {
            unsafe { &*self.0 }._unref();
        }
    }

    /// A trait for types that can be converted to a shared pointer that may be null.
    pub trait ToSharedPointer<N> {
        fn shared_ptr(&self) -> *mut N;
    }

    impl<N: RefCounted> ToSharedPointer<N> for Option<RCHandle<N>> {
        #[inline]
        fn shared_ptr(&self) -> *mut N {
            match self {
                Some(handle) => handle.shared_native(),
                None => ptr::null_mut()
            }
        }
    }

    impl<N: RefCounted> ToSharedPointer<N> for Option<&RCHandle<N>> {
        #[inline]
        fn shared_ptr(&self) -> *mut N {
            match self {
                Some(handle) => handle.shared_native(),
                None => ptr::null_mut()
            }
        }
    }

    /// Clone for bindings types we can not implement Clone for.
    pub trait InternalClone {
        fn clone(&self) -> Self;
    }
}