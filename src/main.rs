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

// fn func_2() {
//     println!("Function 2");
// }

fn  main(){
    let v1 = [1, 2, 3].iter();
    let v2 = ["a", "b", "c"].iter();
    
    for (a, b) in v1.zip(v2){
        println!("{a}, {b}");
    }
}