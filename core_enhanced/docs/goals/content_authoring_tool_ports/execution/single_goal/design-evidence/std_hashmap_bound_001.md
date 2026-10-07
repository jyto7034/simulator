# R03 std HashMap storage bound — root source follow-up001

Rust1.91.1의 [std manifest](https://raw.githubusercontent.com/rust-lang/rust/1.91.1/library/std/Cargo.toml)는
hashbrown0.15를 요청하며 [repository lock](https://raw.githubusercontent.com/rust-lang/rust/1.91.1/Cargo.lock)은0.15.5를 기록한다.
[std HashMap](https://raw.githubusercontent.com/rust-lang/rust/1.91.1/library/std/src/collections/hash/map.rs)의 capacity/Clone은
hashbrown에 위임한다. 설치된 std의 allocator byte introspection은 수행하지 않았다. Fetched byte 해시는 [기록](std_hashmap_bound_001.json)에 보존했다.

[hashbrown0.15.5 raw table](https://raw.githubusercontent.com/rust-lang/hashbrown/v0.15.5/src/raw/mod.rs)의
capacity(1282)·reserve_rehash_inner(2627)·capacity_to_buckets(103)·Clone(3155)을 읽었다. Capacity는 usable 항목 수의 하한이고
tombstone churn에서 감소할 수 있으나 Clone은 원 bucket 수를 복제한다. Current capacity를 그대로 쓰거나 단순2배 하는 것으로
backing bucket 상한을 주장하지 않는다.

이 wrapper의 map은 empty 생성 이후 insert/remove/clear만 사용한다. Lifetime insertion count P를 clear 후에도 보존하면
재할당 시 요청 항목 수가2P 이하라는 분기로부터 B=0(P=0), 그 외 max(16,next_power_of_two((2P×8)/7))가 bucket 상한이다.
이 계산은 checked arithmetic으로 수행한다. 동일식은 별도 검증된 Parry의 hashbrown0.16 pair table에도 적용한다.
새 reserve/import/shrink 경로·표준 라이브러리 구현 변경은 이 source model의 재검증 대상이다. 정확한 allocator/RSS byte 증명이 아니다.
