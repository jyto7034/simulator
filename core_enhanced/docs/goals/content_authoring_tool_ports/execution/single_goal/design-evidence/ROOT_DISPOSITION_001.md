# Root control-design disposition 001

두 helper 원문은 byte-exact 보존했다. Root가 보고서 1–102행 전체를 읽었다. 별도 helper/no new tests이며 완료 감사가 아니다.

채택할 기술 근거: request-local shared accounting은 spawn/summon/boss clone rollback에도 fuel과 first failure를 유지해야 한다.
Loader generated setup 이전부터 scope를 설치하고 actual B time-zero 및 mandatory result 준비 직전까지 유지한다.
Same-time bucket·queue·effects/targets/repeats·deferred/entity·constructor/clones의 선행 admission과 fallible polls가 필요하다.
Infallible helper의 fake identity/sentinel은 채택하지 않는다. 실제 expect/Ok(Rejected)/context 복구 경계를 별도 상세화한다.
TLS는 현재 synchronous route에서 가능한 구현 선택이며 nested authoring reset 금지·RAII 복구·borrow를 callback 위로 유지하지 않음과
ambient reentry disposition을 구현해야 한다. Rapier 내부 중단·OS I/O/OOM/CPU deadline을 보장하지 않는다.

명시적으로 거절한 문장: helper의 Candidate/session lifetime 및 마지막 terminal oracle가 optional trace overflow까지 B rollback으로
묶었다. 이는 accepted CRA P05/DR01/Goal I04와 충돌한다. Optional trace count/byte overflow는 gap만 표시하며 동일 mandatory
admission/commit/continuation/RNG를 유지한다. Mandatory result/receipt/control 실패만 candidate를 폐기한다.
보고서 원문을 고쳐 과거 판단을 숨기지 않고 이 root disposition이 실제 구현 소비 범위를 한정한다.

35개 source hash의 current 비교 결과는 같은 directory의 ROOT_SOURCE_CURRENTNESS_001.json이다.
