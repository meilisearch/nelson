use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub struct Stub<'a, A, R> {
    name: String,
    times: Mutex<Option<usize>>,
    stub: Box<dyn FnMut(A) -> R + Send + 'a>,
}

impl<'a, A, R> Drop for Stub<'a, A, R> {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            let lock = self.times.lock().unwrap();
            if let Some(n) = *lock {
                assert_eq!(n, 0, "{} not called enough times", self.name);
            }
        }
    }
}

impl<'a, A, R> Stub<'a, A, R> {
    pub fn call(&mut self, args: A) -> R {
        let mut lock = self.times.lock().unwrap();
        match *lock {
            Some(0) => panic!("{} called to many times", self.name),
            Some(ref mut times) => {
                *times -= 1;
            }
            None => (),
        }

        (self.stub)(args)
    }
}

#[derive(Default)]
struct StubStore {
    inner: Arc<Mutex<HashMap<String, (Box<Stub<'static, (), ()>>, &'static str)>>>,
}

impl StubStore {
    pub fn insert<'a, A, R>(&'a self, name: String, stub: Stub<'a, A, R>) {
        let mut lock = self.inner.lock().unwrap();
        let stub = unsafe { std::mem::transmute(stub) };
        let ty = std::any::type_name::<(A, R)>();

        lock.insert(name, (Box::new(stub), ty));
    }

    pub unsafe fn get<'a, A, B>(&'a self, name: &str) -> Option<&mut Stub<'a, A, B>> {
        let mut lock = self.inner.lock().unwrap();
        match lock.get_mut(name) {
            Some((s, stored_ty)) => {
                let ty = std::any::type_name::<(A, B)>();
                assert_eq!(
                    ty, *stored_ty,
                    "{} called with unexpected type:\n expected {}, found {} instead.",
                    name, stored_ty, ty
                );
                let s = s.as_mut() as *mut _ as *mut Stub<'a, A, B>;
                Some(&mut *s)
            }
            None => None,
        }
    }
}

pub struct StubBuilder<'a, A, R> {
    name: String,
    store: &'a StubStore,
    times: Option<usize>,
    _f: std::marker::PhantomData<fn(A) -> R>,
}

impl<'a, A: 'static, R: 'static> StubBuilder<'a, A, R> {
    /// Asserts the stub has been called exactly `times` times.
    #[must_use]
    pub fn times(mut self, times: usize) -> Self {
        self.times = Some(times);
        self
    }

    /// Asserts the stub has been called exactly once.
    #[must_use]
    pub fn once(mut self) -> Self {
        self.times = Some(1);
        self
    }

    /// The function that will be called when the stub is called. This needs to be called to
    /// actually build the stub and register it to the stub store.
    pub fn then(self, f: impl FnMut(A) -> R + Sync + Send + 'static) {
        let times = Mutex::new(self.times);
        let stub = Stub {
            stub: Box::new(f),
            times,
            name: self.name.clone(),
        };

        self.store.insert(self.name, stub);
    }
}

/// Mocker allows to stub metod call on any struct. you can register stubs by calling
/// `Mocker::when` and retrieve it in the proxy implementation when with `Mocker::get`.
#[derive(Default)]
pub struct Mocker {
    store: StubStore,
}

impl Mocker {
    pub fn when<A, R>(&self, name: &str) -> StubBuilder<A, R> {
        StubBuilder {
            name: name.to_string(),
            store: &self.store,
            times: None,
            _f: std::marker::PhantomData,
        }
    }

    pub unsafe fn get<'a, A, R>(&'a self, name: &str) -> &mut Stub<'a, A, R> {
        match self.store.get(name) {
            Some(stub) => stub,
            None => panic!("unexpected call to {}", name),
        }
    }
}
