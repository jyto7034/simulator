// macro_rules! dispatch_to {
//     ($val:expr => {$($func:ident),*}) => {
//             match $val {
//                 $(
//                     stringify!($func) => $func(),
//                 )*
//                 _ => {},
//             }
//     }
// }

// fn main() {
//     let s: String = "func_1".into();

//     dispatch(&s);
// }

// fn dispatch(s: &str) {
//     dispatch_to!(s => {func_1, func_2});
// }

// fn func_1(){
//     println!("Function 1");
// }

use std::{
    borrow::BorrowMut,
    cell::RefCell,
    rc::{Rc, Weak},
    task::Waker,
};

// fn func_2() {
//     println!("Function 2");
// }
#[derive(Debug)]
struct Other {
    id: usize,
    base: Option<Weak<RefCell<Base>>>,
}

#[derive(Debug)]
struct Base {
    other: Option<Weak<RefCell<Other>>>,
    id: usize,
}

fn main() {
    let o = Rc::new(RefCell::new(Other { id: 0, base: None }));

    let b = Rc::new(RefCell::new(Base {
        id: 1,
        other: Some(Rc::downgrade(&o)),
    }));

    o.as_ref().borrow_mut().base = Some(Rc::downgrade(&b));
}
