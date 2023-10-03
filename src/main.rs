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

use std::{cell::RefCell, rc::Rc, rc::Weak};

struct Player {
    opp: Option<Rc<RefCell<Player>>>,
    data: i32,
}

impl Player {
    pub fn set(&mut self, new_opponent: &Option<Weak<RefCell<Player>>>) {
        if let Some(data) = new_opponent.as_ref().unwrap().upgrade() {
            self.opp = Some(Rc::clone(&data));
        }
    }
}

fn main() {
    let p1 = Rc::new(RefCell::new(Player {
        opp: None,
        data: 10,
    }));

    let p2 = Rc::new(RefCell::new(Player {
        opp: None,
        data: 12,
    }));

    p1.as_ref().borrow_mut().set(&Some(Rc::downgrade(&p2)));
    p2.as_ref().borrow_mut().set(&Some(Rc::downgrade(&p1)));

    let d = p1
        .as_ref()
        .borrow_mut()
        .opp
        .as_ref()
        .unwrap()
        .as_ref()
        .borrow_mut()
        .data;
    println!("{}", d);
}
