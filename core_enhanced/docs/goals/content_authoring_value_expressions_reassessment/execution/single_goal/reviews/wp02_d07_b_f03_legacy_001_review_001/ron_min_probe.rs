// Read-only review probe against the workspace's already built RON dependency.
fn main() {
    for value in [i32::MIN, i32::MIN + 1, -7, 0, 7, i32::MAX] {
        let serialized = ron::ser::to_string(&value).unwrap();
        let parsed = ron::de::from_str::<i32>(&serialized);
        println!("value={value} serialized={serialized} parsed={parsed:?}");
        if value == i32::MIN {
            assert!(parsed.is_err());
            assert_eq!(ron::de::from_str::<i64>(&serialized).unwrap(), i64::from(value));
        } else {
            assert_eq!(parsed.unwrap(), value);
        }
    }
}
