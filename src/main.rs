fn main() {
    let a: Vec<&mut i32> = vec![];
    let b: &mut Vec<&i32> = &mut vec![&1, &2, &3];
    // a.push(&mut 3);
    b[0] = &3;
    println!("{:#?}", b);
}
