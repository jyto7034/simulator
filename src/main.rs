fn main() {
    // 두 개의 Vec<String> 객체 생성
    let vec1: Vec<String> = vec!["apple", "banana", "cherry", "fig"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let vec2: Vec<String> = vec!["banana", "date", "fig"]
        .iter()
        .map(|s| s.to_string())
        .collect();

    // 겹치지 않는 원소를 찾아서 unique_elements에 저장
    let unique_elements: Vec<String> = vec1
        .iter()
        .cloned()
        .filter(|element| !vec2.contains(element))
        .chain(vec2.iter().cloned().filter(|element| !vec1.contains(element)))
        .collect();

    // 결과 출력
    println!("겹치지 않는 원소들: {:?}", unique_elements);
}
