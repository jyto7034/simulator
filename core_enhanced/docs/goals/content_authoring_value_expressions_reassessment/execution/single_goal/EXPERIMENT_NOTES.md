# R06 GEX-10 기술 메모

## Current decisions

- 첫 검토 경계는 `R06_STAGE_REVIEW_TARGET_001`의 typed expression kernel으로 확정했다. Goal/WP를 독립화하지 않으며 in-memory AST→bounded compile→immutable segmented stack plan→checked evaluation까지 완결하고, authored RON/catalog/runtime에는 아직 노출하지 않아 기존 literal production 동작을 보존한다.
- RON 0.5.1의 `ron::de::Deserializer`는 `DeserializeSeed`를 호출할 수 있으나 parse depth option을 제공하지 않는다. recursive `ValueExpr: Deserialize` derive를 먼저 공개하지 않는다. authored 연결 구간에서 node를 읽기 전에 depth/node budget을 차감하는 custom visitor를 사용한다.
- compiler limits 초기값은 CRA D08의 technical safety admission(depth 64, nodes 1024, stack 1024, work 65536)을 넘지 않는다. 기존 global execution limits와 결합은 WP02/publication 구간에서 더 작은 상한을 선택한다.
- aggregate는 member/group/work/retained를 별도 charge하며 nested worst-case member product를 compile 때 checked admission한다. owner key는 `Side/Employee/EquipmentCopy` domain을 보존하고 한 aggregate의 domain 혼합을 거부한다.
- semantic fingerprint는 RON serializer에 의존하지 않는 versioned canonical binary encoding을 사용한다. RON 0.5.1은 i128 serialization을 지원하지 않아 typed AST 자체를 RON으로 직렬화하는 방식은 사용할 수 없다.
- runtime result 읽기는 `query_action`을 settlement 증거로 재사용할 수 없다. current `take_settled_action_result`, `FactSummary`, action handle을 바탕으로 effect-prefix witness를 새로 연결해야 한다.
- 현재 `with_instant_resource_delivery` callback은 `FnOnce(&mut Self)`이고 close 오류를 성공으로 묻는 구조다. WP02가 Result propagation과 candidate rollback을 소유한다.

## Remaining scope after the first kernel checkpoint

- DamageExpr/HealExpr authored custom bounded deserialization, EffectSite/capability validation, compiled-plan catalog와 publication diagnostics.
- runtime EvalFrame, snapshot/live carrier, actual finite target/grouping, resource lease reads, completed-effect prefix, sequential execution and failure propagation.
- Attack/SpellPower/weapon/copy enhancement/cadence composition, zero omission, old V7/nonzero round-trip and Certified fingerprint.
- public pack/CLI/simulate evidence, canonical contract documents, V01–V21 integration/regression/cost evidence and later independent GEX-20 audit.

## Open findings

현재 새 policy 결정이나 scope/권한 blocker는 없다. 전체 repo Clippy의 기존 deny 3건과 repo-wide rustfmt 차이는 initial baseline bytes와 일치하는 별도 작업 트리 상태다. parser hook, callback propagation, prefix owner와 zero omission은 계획에 이미 남은 기술 준비 항목이며 각 owning 구간 전 실제 interfaces에 맞춰 닫아야 한다.
