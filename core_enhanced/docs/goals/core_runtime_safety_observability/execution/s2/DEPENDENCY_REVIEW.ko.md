# S2 dependency source review — pre-mutation

실제 crates.io 다운로드와 source/metadata를 확인했다. [입력·전체 graph](evidence/dependencies/REVIEW_INPUTS.json), metadata.json, probe Cargo.lock 및3개 crate source archive에 SHA256을 기록했다. 이 검토용 별도 workspace는 제품 Cargo.toml/lock을 변경하지 않았다. 실제 제품 통합 뒤 선택된 graph는 다시 확인한다.

| Crate | Selected exact version | Declared Rust | License |
| --- | --- | --- | --- |
| metrics | 0.24.6 | 1.71.1 | MIT |
| metrics-exporter-prometheus | 0.18.3 | 1.71.1 | MIT AND Apache-2.0 |
| metrics-util (resolved transitive) | 0.20.4 | 1.71.1 | MIT |

현재 도구는 rustc1.98.1이다. 전체106개 package의 버전·Rust·license 원문은 REVIEW_INPUTS에 있다. 직접 두 dependency는 `=`로 고정하고 root Cargo.lock이 전이 patch를 고정한다. exporter default-features=false, http-listener만 선택한다. Push gateway/TLS/protobuf/native histogram 기능을 켜지 않는다. crate 자체의 declared MSRV를 전체 graph의 MSRV로 오인하지 않는다.

[공식 metrics API](https://docs.rs/metrics/0.24.6/metrics/)의 Recorder/Counter/Gauge/Histogram을 사용하고 [표준 exporter](https://docs.rs/metrics-exporter-prometheus/0.18.3/metrics_exporter_prometheus/)가 Prometheus format과 HTTP를 소유한다. 등록은 bootstrap의 owned recorder에서 유한 key/label 집합으로 수행하며 global recorder를 영구 설치할 필요가 없다. 게임 hot path에는 metric 등록·문자열 formatting·histogram record가 없어야 한다.

실제 source 확인:

- exporter `registry.rs`: histogram은 metrics-util AtomicBucket<(f64, quanta::Instant)>를 감싼다. 무제한 direct record를 안전하다고 주장할 수 없다.
- exporter `recorder.rs:124–139,312–314`: run_upkeep은 histogram raw samples를 clear_with로 drain하여 distribution에 합친다. Worker만 유한 batch를 기록한 뒤 즉시 동기 run_upkeep을 호출한다. Worker가 정지하면 추가 histogram 기록도 정지하며 bounded Core queue가 포화된 뒤 관측만 drop한다.
- exporter `distribution.rs`와 metrics-util `storage/histogram.rs`: 명시적 finite buckets는 고정 bounds/counts/sum 형태의 classic Histogram을 사용한다. 기본 rolling Summary 대신 이 경로를 선택한다. scrape 없는 idle에도 worker upkeep을 유지한다.
- metrics-util `storage/bucket.rs`: block은64-bit에서64 samples이며 clear_with는 epoch 기반으로 retire하고 flush한다. 논리적 미소비 raw 표본 상한과 실제 heap reclamation을 구분해야 한다. 지연 해제는 기존 epoch reader의 quiescence에 의존하므로 즉시 free된다고 주장하지 않는다. 동일 collector runtime의 고정 task 수와 record/upkeep ordering, 실제 반복·idle heap 회수 검증을 storage 증거에 포함한다. 이 경계의 실제 상한 증명은 구현 후 검증 owner에 남아 있다.
- builder `build`는 Tokio runtime 안에서 호출하며 내부 upkeep task도 spawn한다. 전용 collector runtime의 생명주기로 해당 task/HTTP를 함께 정리하고 수동 batch upkeep을 대체하지 않는다. build_recorder만 쓴 테스트는 수동 upkeep의 책임을 직접 가진다.

Core feature 이름, queue/batch/series 수, 정확한 lifecycle/selection API와 storage bounds는 IMPLEMENTATION_DECISIONS에서 확정한 뒤 제품 변경한다. 동결 TD05–07 범위의 기술 선택이며 정책 수치나 gameplay 의미를 변경하지 않는다.

추가 retirement gate 검토: crossbeam-epoch0.9.21 (MIT OR Apache-2.0, declared Rust1.61)를 metrics-util과 같은 default collector로 직접 사용한다. exact probe source/lock/archive는 evidence/dependencies/epoch_probe001에 있다. 실제 s2_histogram_epoch_probe001에서 pinned reader 동안10,000재시도 live heap22123→22123bytes, 해제 후4000×128표본과 선행256표본의 count512256, idle heap17931bytes를 확인했다. 이 probe 결과는 제품 V10 전체 통과가 아니다. 구현 결정과 동시 collector의 in-flight callback 상한은 IMPLEMENTATION_DECISIONS에 기록했다.

## Integrated source review

제품 통합 graph는 [product002/REVIEW.json](evidence/dependencies/product002/REVIEW.json) 및 같은 폴더의 metadata/lock에 기록했다. S1의311개 name/version 쌍은 모두 보존되고32개가 추가되어343개다. Direct optional dependency3개를 exact pin하며 metrics-util0.20.4 및 전체 전이는 root lock으로 고정했다. 실제 rustc1.98.1에서 locked/offline debug/release feature 조합을 검증한다. 각 package의 declared Rust/license는 metadata 원문이며 전체 graph의 최소 지원 compiler를 추정하지 않는다. `http-listener`와 exporter가 요구하는 runtime/server feature만 사용한다. 실제 product worker의 allocator/upkeep 증거는 s2_exporter_heap_test001 및 최종 재검증 receipt를 따른다.

로컬 외부 소비자는 공식 Prometheus3.15.0/Grafana OSS13.2.3 바이너리의 SHA256을 공식 다운로드 목록과 대조했다. 이들은 제품 Cargo dependency나 자동 다운로드가 아니며, 명시적으로 지정한 로컬 실행 파일이다. 다운로드 출처/크기/hash는 evidence/services/*download001.json에 있다. Playwright/Chromium은 브라우저 검증 도구이며 제품 dependency가 아니다. 실제 query/화면/버전은 각 live*/browser.json과 screenshot에 보존한다.

Grafana13.2.3 XY schema와 실제 source는 evidence/services/{xychart-panelcfg.gen.ts,utils.ts,migrations.ts}에 보존했다. Manual mapping은 frame/x/y matcher가 모두 필요하며 `pluginVersion`이 비어 있으면 구버전 migration이 matcher 객체를 다시 감싼다. 최종 panel은 pluginVersion13.2.3과 frame index0을 명시한다. 실제 서비스/브라우저 검증을 대신해 JSON schema만 보고 표시 성공으로 판단하지 않는다.
