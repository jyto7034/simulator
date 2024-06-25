fn main() {
    let mut v1 = vec![1, 2, 3, 4];
    let r = v1.iter().position(|it| it == &2);
    println!("{:#?}", r);
}
