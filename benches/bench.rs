use card_game::test::spawn_server;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_spawn_server(c: &mut Criterion) {
    c.bench_function("spawn_server", |b| {
        b.iter(|| {
            // spawn_server는 이미 tokio::spawn 내부에서 서버를 실행하므로,
            // 별도의 block_on 없이 직접 호출합니다.
            black_box(spawn_server());
        })
    });
}

criterion_group!(benches, bench_spawn_server);
criterion_main!(benches);