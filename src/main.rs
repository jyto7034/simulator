fn main() {
    fn no_fun(x: f32, f: impl Fn(f32) -> f32) -> f32 {
        f(x)
    }

    let a = 3.;
    let x = 2.;
    let f = |x| x * a;

    println!("Product of {} and {} is {}", x, a, no_fun(x, f));
}
