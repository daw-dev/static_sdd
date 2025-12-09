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

// enum Pass<T> {
//     Ready {
//         ref_callbacks: Vec<Box<dyn FnOnce(&T)>>,
//         move_callback: Option<Box<dyn FnOnce(T)>>,
//     },
//     Passed,
// }
//
// impl<T: 'static> Pass<T> {
//     fn pass(&mut self, value: T) {
//         match self {
//             Self::Ready {
//                 ref_callbacks,
//                 move_callback,
//             } => {
//                 for callback in std::mem::take(ref_callbacks).into_iter() {
//                     callback(&value);
//                 }
//                 if let Some(callback) = move_callback.take() {
//                     callback(value);
//                 }
//                 *self = Self::Passed;
//             }
//             Self::Passed => panic!("Value already passed!"),
//         }
//     }
// }
//
// enum Waiting<T> {
//     Empty,
//     Computed(T),
// }
//
// impl<T> Waiting<T> {
//     fn set(&mut self, value: T) {
//         *self = Self::Computed(value);
//     }
// }
//
// enum Internal<T> {
//     Pass(Pass<T>),
//     Waiting(Waiting<T>),
// }
//
// impl<T: 'static> Internal<T> {
//     fn new() -> Self {
//         Self::Waiting(Waiting::Empty)
//     }
//
//     // fn one_hop() -> (Self, Self) {
//     //     let read = Self::new();
//     //     let write = Self::pass(read);
//     //     (write, read)
//     // }
//
//     fn set(&mut self, value: T) {
//         match self {
//             Self::Pass(pass) => pass.pass(value),
//             Self::Waiting(waiting) => waiting.set(value),
//         }
//     }
//
//     fn pass(mut other: Internal<T>) -> Self {
//         Self::Pass(Pass::Ready {
//             ref_callbacks: Vec::new(),
//             move_callback: Some(Box::new(move |val| other.set(val))),
//         })
//     }
//
//     fn pass_multiple(others: impl IntoIterator<Item = Internal<T>>) -> Self
//     where
//         T: Clone,
//     {
//         Self::Pass(Pass::Ready {
//             ref_callbacks: others
//                 .into_iter()
//                 .map(|mut internal| {
//                     Box::new(move |val: &T| internal.set(val.clone())) as Box<dyn FnOnce(&T)>
//                 })
//                 .collect(),
//             move_callback: None,
//         })
//     }
//
//     fn pass_map<U, F>(mut other: Internal<U>, mapper: F) -> Internal<T>
//     where
//         U: 'static,
//         F: FnOnce(T) -> U + 'static,
//     {
//         Internal::Pass(Pass::Ready {
//             ref_callbacks: Vec::new(),
//             move_callback: Some(Box::new(move |val| other.set(mapper(val)))),
//         })
//     }
//
//     fn pass_multiple_ref<U, F>(
//         others: impl IntoIterator<Item = Internal<U>>,
//         mapper: F,
//     ) -> Internal<T>
//     where
//         U: 'static,
//         F: FnOnce(&T) -> U + 'static,
//     {
//         Self::Pass(Pass::Ready {
//             ref_callbacks: others
//                 .into_iter()
//                 .map(|mut internal| {
//                     Box::new(move |val: &T| internal.set(mapper(val))) as Box<dyn FnOnce(&T)>
//                 })
//                 .collect(),
//             move_callback: None,
//         })
//     }
// }
//
// pub struct Inherited2<T> {
//     internal: Internal<T>,
// }
//
// pub struct Deferred2<T> {
//     internal: Internal<T>,
// }

struct PassDown<T, U> {
    next_step: Box<State<T, U>>,
    mapper: Box<dyn FnOnce(T) -> T>,
}

struct PassUp<T> {
    mapper: Box<dyn FnOnce(T) -> T>,
}

struct Internal<T, U> {
    pass_down: PassDown<T, U>,
    pass_up: PassUp<U>,
}

struct Leaf<T, U> {
    mapper: Box<dyn FnOnce(T) -> U>,
}

enum State<T, U = T> {
    Leaf(Leaf<T, U>),
    Internal(Internal<T, U>),
}

impl<T: 'static, U: 'static> State<T, U> {
    fn new() -> Self {
        todo!()
    }

    fn hop_map<F>(mapper: F) -> Self
    where
        F: FnOnce(T) -> U + 'static,
    {
        Self::Leaf(Leaf {
            mapper: Box::new(mapper),
        })
    }

    fn into_set(self, value: T) -> U {
        match self {
            State::Internal(Internal { pass_down, pass_up }) => {
                (pass_up.mapper)(pass_down.next_step.into_set((pass_down.mapper)(value)))
            }
            State::Leaf(hop) => (hop.mapper)(value),
        }
    }
}

impl<T: 'static> State<T, T> {
    fn hop() -> Self {
        Self::hop_map(|t| t)
    }
}

pub struct Inherited2<T> {
    internal: State<T>,
}

pub struct Deferred2<T> {
    internal: State<T>,
}
