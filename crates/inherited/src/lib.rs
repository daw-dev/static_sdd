use std::cell::RefCell;
use std::rc::Rc;

struct SharedState<T> {
    value: Option<T>,
    callbacks: Vec<Box<dyn FnOnce()>>,
}

pub struct Inherited<T> {
    state: Rc<RefCell<SharedState<T>>>,
}

impl<T> Clone for Inherited<T> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}

impl<T: 'static> Inherited<T> {
    pub fn new() -> Self {
        Self {
            state: Rc::new(RefCell::new(SharedState {
                value: None,
                callbacks: Vec::new(),
            })),
        }
    }

    pub fn set(&mut self, val: T) {
        let mut state = self.state.borrow_mut();
        if state.value.is_some() {
            panic!("Value set twice!");
        }
        state.value = Some(val);

        let callbacks = std::mem::take(&mut state.callbacks);
        drop(state);

        for cb in callbacks {
            cb();
        }
    }

    pub fn inherit(mut other_promise: Self) -> Self {
        let res = Self::new();
        let my_state = res.state.clone();

        res.register_callback(move || {
            let val = my_state
                .borrow_mut()
                .value
                .take()
                .expect("The value was already moved, consider using inherit_cloned");
            other_promise.set(val);
        });
        res
    }

    pub fn inherit_multiple(other_inheriteds: impl IntoIterator<Item = Self> + 'static) -> Self
    where
        T: Clone,
    {
        let res = Self::new();
        let my_state = res.state.clone();

        res.register_callback(move || {
            let val = my_state
                .borrow_mut()
                .value
                .take()
                .expect("The value was already moved, consider using inherit_cloned");

            other_inheriteds
                .into_iter()
                .for_each(|mut inh| inh.set(val.clone()));
        });
        res
    }

    pub fn inherit_cloned(mut other_promise: Self) -> Self
    where
        T: Clone,
    {
        let res = Self::new();
        let my_state = res.state.clone();

        res.register_callback(move || {
            let val = my_state
                .borrow_mut()
                .value
                .clone()
                .expect("The value was already moved");
            other_promise.set(val);
        });
        res
    }

    pub fn inherit_map<U, F>(mut other_promise: Inherited<U>, mapper: F) -> Self
    where
        U: 'static,
        F: FnOnce(T) -> U + 'static,
    {
        let res = Self::new();
        let my_state = res.state.clone();

        res.register_callback(move || {
            let val = my_state.borrow_mut().value.take().expect(
                "The value was already moved, consider using inherit_map_ref and cloning the value",
            );
            other_promise.set(mapper(val));
        });
        res
    }

    pub fn inherit_map_ref<U, F>(mut other_promise: Inherited<U>, mapper: F) -> Self
    where
        U: 'static,
        F: FnOnce(&T) -> U + 'static,
    {
        let res = Self::new();
        let my_state = res.state.clone();

        res.register_callback(move || {
            let borrow = my_state.borrow();
            let val = borrow.value.as_ref().expect("The value was already moved");
            other_promise.set(mapper(val));
        });
        res
    }

    pub fn channel() -> (Self, Deferred<T>) {
        let out = Deferred::new();
        let mut out_clone = out.clone();
        let inp = Self::new();
        let my_state = inp.state.clone();

        inp.register_callback(move || {
            let val = my_state
                .borrow_mut()
                .value
                .take()
                .expect("The value was already moved");
            out_clone.set(val);
        });
        (inp, out)
    }

    pub fn channel_map<U, F>(mapper: F) -> (Self, Deferred<U>)
    where
        U: 'static,
        F: FnOnce(T) -> U + 'static,
    {
        let out = Deferred::new();
        let mut out_clone = out.clone();
        let inp = Self::new();
        let my_state = inp.state.clone();

        inp.register_callback(move || {
            let val = my_state
                .borrow_mut()
                .value
                .take()
                .expect("The value was already moved");
            out_clone.set(mapper(val));
        });
        (inp, out)
    }

    pub fn channel_map_ref<U, F>(mapper: F) -> (Self, Deferred<U>)
    where
        U: 'static,
        F: FnOnce(&T) -> U + 'static,
    {
        let out = Deferred::new();
        let mut out_clone = out.clone();
        let inp = Self::new();
        let my_state = inp.state.clone();

        inp.register_callback(move || {
            let borrow = my_state.borrow_mut();
            let val = borrow.value.as_ref().expect("The value was already moved");
            out_clone.set(mapper(val));
        });
        (inp, out)
    }

    fn register_callback<F: 'static + FnOnce()>(&self, cb: F) {
        let mut state = self.state.borrow_mut();
        if state.value.is_some() {
            drop(state);
            cb();
        } else {
            state.callbacks.push(Box::new(cb));
        }
    }
}

pub struct Deferred<T> {
    state: Rc<RefCell<SharedState<T>>>,
}

impl<T> Clone for Deferred<T> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}

impl<T: 'static> Deferred<T> {
    pub fn new() -> Self {
        Self {
            state: Rc::new(RefCell::new(SharedState {
                value: None,
                callbacks: Vec::new(),
            })),
        }
    }

    fn set(&mut self, val: T) {
        let mut state = self.state.borrow_mut();
        if state.value.is_some() {
            panic!("Value set twice!");
        }
        state.value = Some(val);

        let callbacks = std::mem::take(&mut state.callbacks);
        drop(state);

        for cb in callbacks {
            cb();
        }
    }

    pub fn unwrap(&self) -> T {
        self.state
            .borrow_mut()
            .value
            .take()
            .expect("The value was not yet resolved or already moved")
    }

    pub fn map<U: 'static, F>(&self, mapper: F) -> Deferred<U>
    where
        F: 'static + FnOnce(T) -> U,
    {
        let res = Deferred::new();
        let mut res_clone = res.clone();

        let my_state = self.state.clone();

        self.register_callback(move || {
            let val = my_state.borrow_mut().value.take().expect(
                "The value was already moved, consider using map_ref and cloning the value",
            );

            res_clone.set(mapper(val));
        });

        res
    }

    pub fn map_ref<U: 'static, F>(&self, mapper: F) -> Deferred<U>
    where
        F: 'static + FnOnce(&T) -> U,
    {
        let res = Deferred::new();
        let mut res_clone = res.clone();
        let my_state = self.state.clone();

        self.register_callback(move || {
            let new_val = mapper(
                my_state
                    .borrow()
                    .value
                    .as_ref()
                    .expect("The value was already moved"),
            );
            res_clone.set(new_val);
        });

        res
    }

    fn register_callback<F: 'static + FnOnce()>(&self, cb: F) {
        let mut state = self.state.borrow_mut();
        if state.value.is_some() {
            drop(state);
            cb();
        } else {
            state.callbacks.push(Box::new(cb));
        }
    }
}

pub trait PassDown<TPrime = Self> {
    fn pass_down(self) -> TPrime;
}

pub trait PassUp<U> {
    fn pass_up(self) -> U;
}

pub struct InheritOnce<T, U = T> {
    mapper: Box<dyn FnOnce(T) -> U>,
}

impl<T: 'static, U: 'static> InheritOnce<T, U> {
    pub fn inherit(other_inherited: Self) -> Self {
        other_inherited
    }

    pub fn also<F>(self, clos: F) -> Self
    where
        F: FnOnce(&T) + 'static,
    {
        InheritOnce {
            mapper: Box::new(|t| {
                clos(&t);
                self.resolve(t)
            }),
        }
    }

    pub fn pass_down_with<T1, F>(self, pass_down: F) -> InheritOnce<T1, U>
    where
        T1: 'static,
        F: FnOnce(T1) -> T + 'static,
    {
        InheritOnce {
            mapper: Box::new(|t| self.resolve(pass_down(t))),
        }
    }

    pub fn pass_up_with<U1, G>(self, pass_up: G) -> InheritOnce<T, U1>
    where
        U1: 'static,
        G: FnOnce(U) -> U1 + 'static,
    {
        InheritOnce {
            mapper: Box::new(|t| pass_up(self.resolve(t))),
        }
    }

    pub fn pass_down<T1>(self) -> InheritOnce<T1, U>
    where
        T1: PassDown<T>,
    {
        InheritOnce {
            mapper: Box::new(|t| self.resolve(t.pass_down())),
        }
    }

    pub fn pass_up<U1>(self) -> InheritOnce<T, U1>
    where
        U: PassUp<U1>,
    {
        InheritOnce {
            mapper: Box::new(|t| self.resolve(t).pass_up()),
        }
    }

    pub fn inherit_default<TPrime, UPrime>(other_inherited: InheritOnce<TPrime, UPrime>) -> Self
    where
        TPrime: 'static,
        UPrime: 'static,
        T: PassDown<TPrime>,
        UPrime: PassUp<U>,
    {
        other_inherited.pass_down().pass_up()
    }

    pub fn base_map<F>(mapper: F) -> Self
    where
        F: FnOnce(T) -> U + 'static,
    {
        Self {
            mapper: Box::new(mapper),
        }
    }

    pub fn resolve(self, value: T) -> U {
        (self.mapper)(value)
    }

    pub fn zip<T1, U1>(self, other: InheritOnce<T1, U1>) -> InheritOnce<(T, T1), (U, U1)>
    where
        T1: 'static,
        U1: 'static,
    {
        InheritOnce {
            mapper: Box::new(|(t, t1)| (self.resolve(t), other.resolve(t1))),
        }
    }
}
