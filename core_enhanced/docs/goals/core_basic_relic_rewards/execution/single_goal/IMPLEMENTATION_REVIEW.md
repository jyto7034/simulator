# U16-2 구현자 자체 검토

이 문서는 GEX-10의 구현자 검토이며 독립 감사나 Complete 판정이 아니다. 실제 baseline과의 소스·시험·데이터 변경24파일은 `SOURCE_DELTA_REVIEW.json`과 `runs/source_delta_review.diff`에 고정했다. 새3파일의 전체 구현과 기존 producer/consumer의 변경 부분, 원시 실패와 교정 결과를 직접 대조했다.

## 계약과 실제 연결

- `relic_reward.rs`의 typed 작성 정책은 증가하는 양수 누적 비용, 양수/중복 없는 후보, u32 가중치 합계와 존재하는 common/rare/special 정의를 검사한다. LiveRon이 실제 publication에서 호출하고, 모든 reward_activity에 Reward category를 요구한다. authoring002의 정상1/거절10이 실제 패키저를 통과했다. 기존 다른 용도의 유물 rarity는 제한하지 않는다.
- U13 배정/예약/metadata를 그대로 소비한다. 세 기본 유형만 실행 지원하며 나머지2유형은 거절한다. 공식 early_reward는 지원되지만 빈 공식 catalog를 다른 보상 pool로 대체하지 않는다. 공식 경로의 빈 선택/사유/퇴장과 기존 일반 보상 pool의 획득을 별도로 검증했다.
- 후보 추첨은 validated pool의 가중치로 종류 중복 없이 최대3개다. 세션은 현재 후보와 선택, inline 투자 등급/누적 지불/입력 UUID만 소유한다. node seed와 정규화한 입력을 사용하고 미래 후보를 조회에 싣지 않는다. 매번 카탈로그를 후보 수만큼 선형 검색하는 대신 일시 BTreeMap 색인을 만든다.
- 투자 연산은 다음 후보 존재/가루 부족을 먼저 판정하고 전액 차감과 후보 교체를 함께 반영한다. 이전 선택은 비우고 이전 UUID를 거절한다. 일반/희귀/특급 차액3/6/9와 후보0·잔량2·최고등급·미이용 무환불을 검증했다.
- 합성은 서로 다른 미장착 사본2개를 선택하고 입력 순서를 정규화한다. 같은 정의의 다른 사본과 Corrupt를 허용한다. 입력 종류를 제외한 뒤 양수 확률의 모든 등급 후보를 검사한다. 고정6행의 합계100/0%배제와 다른 보유 종류 허용을 수작업 기대값으로 검증했다. 확정은 EconomyGrantState와 기존 transaction에서 제거/Normal 지급/노드완료를 묶는다.
- 정상 명령3개는 ActionKind·PlayerBehavior·ApplicationCommandKind·owner·route·capability·실행·결과 projection에 연결된다. 명령 vocabulary84의 bijection/route/owner 검사가 통과했다. 일반/전투 Reward의 기존 완료 시점은 유지하며 새 활동만 Claim/Confirm에서 완료한다. 일반 RewardGranted에는 optional relic_activity가 null로 추가된다.
- RewardSession의 optional/default/skip-none 필드는 구 일반 Reward 직렬화를 유지한다. 새 active owner는 기존 relic extent/clone/store preflight에 포함한다. 새 영구 owner나 이중 자산 엔진, 무제한 history를 만들지 않았다. layout 정규화에 기본 정책이 추가되어 기존 콘텐츠의 semantic identity는 달라진다. 과거 manifest를 덮어쓰지 않고 source receipt와 프로필이 같은 successor를 사용했다.

## 복원·원자성과 공개 소비

기본 profile 실제 host20개 새 프로세스/78명령/OS21 여섯 번, 확장 측정 profile3개/29명령/OS21 두 번을 확인했다. 미완료 Continue는 입장 전 후보/가루/사본으로, 완료 Continue는 소비와 결과를 보존한다. SavePoint는 원래 비용과 자산을 복원한다. 실제 journal 실패 뒤 메모리·active·profile 원본이 동일하고 같은 명령의 유효 재시도가 한 번만 지급한다.

투자 중간 명령은 미완료 Continue의 입장 전 payload가 같아 durable publication을 하지 않을 수 있다. 이 지점에 IO 실패를 기대한 진행기 오류는 보존하고, 실제 발행 경계인 최종 수령/합성/미이용 종료/SavePoint에서 확인했다. 이것은 비용만 저장하는 제품 수정으로 우회하지 않았다. Corrupt 사본을 정상 공개 명령으로 만든 추가 host 경로에서도 역순 입력의 Normal 출력과 새 프로세스 결과 보존을 확인했다.

Core 고유2196은 lib2114+교정9+통합72+동일 소켓1의 조합이며 한 번의 전체 all-targets 녹색 실행이 아니다. 기존 ignore6은 유지했다. `BASELINE_REUSE.json`에 소켓·native4·server31의 exact 원문과 비영향 근거를 구분했다. 현재 전체 실패10의 제품/fixture/환경 원인과 모든 필요한 closure를 연결했다.

## 비용과 제한

새 활동은 정상/후보300종·추가64사본에서6개 비용 프로세스/312개 명령·Run조회 창을 측정했다. 확장 명령의 최대 native work715362, 누적 admission2343939bytes, 실제 allocator 증가 peak680174bytes; Run조회 증가 peak816889bytes다. 전체 프로세스 RSS·보유 trace·정적 데이터는 native owner16MiB와 다른 값이다. 모든 표본에서 app/trace value 해제 후 allocator live requested bytes가 시작값과 정확히 같았다.

공유 lifecycle은 동일12+12/768+768 vault 입력과12연산을 현재 코드에서6프로세스 재실행했다. Continue import heap peak 중앙값은 정상252746→252786, 확장800580→800620bytes다. 실제 시간은 원문에 기록하며 최적화 성과나 device SLO로 주장하지 않는다. generic native 값이 null인 부분은 null로 남기고 새 활동의 실제 활성 native 계측과 구별했다.

후보300종 pack은 기본 authoring10M의 GeneratedSetup에서 거절되었다. 입력/한도를 조용히 줄이거나 올리지 않았다. 같은 workload의 전체 비용을 보기 위해 별도 trusted authoring64M 측정 profile을 고정했고 승인 host까지 검증했다. native16MiB와 제품 기본 설정은 그대로다. 따라서 이 stress pack의 기본 authoring 승인이나 공식 콘텐츠 완성을 주장하지 않는다.

## 증거 현재성과 잔여

Prepared host/worker 빌드 이후 제품 실행 코드는 바뀌지 않았다. `cfg(test)` G11 DEFAULT_INPUTS 문자열만 새 manifest로 바뀌었고 default_restore010에서4개가 통과했다. 컴파일 당시 preimage와 비영향 사유를 보존했다. 실제 host 증거는 복사한 binary fingerprint에 정확히 귀속하며, 현재 소스의 새 build fingerprint에 과거 승인을 재사용하지 않는다.

일반 튜닝3/9/18·가중치1은 위임 초기값이다. U16-3/4, 공식 유물 효과 확장/U21, Unity와 최종 SLO는 미실행이다. 임의 저장/checksum/메모리 변조는 제외했다. 모든 현재 완료 조건은 COVERAGE의17수용/10차원/5IVA/2실행노드와 원시 증거에 연결했다. 마지막 격리 정역복원/target 고정이 성공하면 구현·자체검증을 끝내고 별도 GEX-20 요청을 기다린다.
