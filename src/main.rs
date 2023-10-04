enum Zone {
    A(ZoneA),
    B(ZoneB),
}

struct ZoneA {
    data: i32,
}

struct ZoneB {
    data: f64,
}

fn get_obj(condition: bool) -> Zone {
    if condition {
        Zone::A(ZoneA { data: 42 }) // ZoneA 인스턴스 생성
    } else {
        Zone::B(ZoneB { data: 3.14 }) // ZoneB 인스턴스 생성
    }
}

fn main() {
    let obj = get_obj(true);

    match obj {
        Zone::A(zone_a) => {
            println!("Accessing data: {}", zone_a.data);
        }
        Zone::B(zone_b) => {
            println!("Accessing data: {}", zone_b.data);
        }
    }
}
