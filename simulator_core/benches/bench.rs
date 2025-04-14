use criterion::{criterion_group, criterion_main, Criterion};
use simulator_core::test::spawn_server;
use tokio::runtime::Runtime;

fn bench_spawn_server(c: &mut Criterion) {
    // Tokio 런타임 생성
    let rt = Runtime::new().unwrap();

    c.bench_function("spawn_server", |b| {
        b.iter(|| {
            // 런타임 내에서 비동기 작업 실행
            rt.block_on(async {
                let _ = spawn_server().await;
            });
        });
    });
}

criterion_group!(benches, bench_spawn_server);
criterion_main!(benches);
