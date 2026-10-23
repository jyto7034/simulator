# U16-1 판단과 미해결

- P2G10: 초기 상품 기준가(유물60/부활150/침식30)와 기본 부활상품은 CF-D06·기존 가격 조절 위임에서 기록했다. 사용자 직접 수치 답변으로 표기하지 않는다.
- P2G20: 즉시치료 일반 소비품은 이전 modifier를 유지하여 명시 교체 규칙과 불일치. 치료 자체의 수치/조건은 유지하면서 교체를 보완한다.
- P2G40: U16 로드맵의 네 분리 인수 묶음 중 U16-1만 하나의 Single Goal로 구체화. 후속 세 묶음 삭제/전체 U16 완료가 아니다.
- 초기 인계 당시 실제 codec/공개 host/비용 검증은 미수행이었다. 아래 기록과 EXPERIMENTS/COVERAGE가 최종 실행 상태다.

- grant 추적 확인: RelicDefinition.allow_duplicates는 같은 직원의 중복장착 제한이며 중복보유 제한이 아니다(relic.rs grant/validate_employee_slots, relics.md). 기존 grant를 그대로 사용하고 구매에 새 보유 제한을 추가하지 않는다. CODE_REVIEW의 비중복 자격은 이 기존 장착 자격과 구분하여 소비한다. 공식 relic catalog는 아직 비어 있어 정상 외부 representative pack으로 유물 구매를 검증하고 미승인 공식 유물 효과를 추가하지 않는다.


IO 준비001 교정: 미완료 상점 구매는 내부 세션에서 수락하지만 Continue용 export는 입장 전 값을 유지한다. ApprovedPersistence.publish의 checkpoint/profile 동일성 fast path 때문에 purchase에 journal temp 장애를 넣어도 writer로 내려가지 않는다. 호스트는 성공을 실패로 세지 않고 panic했고 실패 store/원문을 보존했다. 제품 문제로 고치지 않는다. 새 격리 분기002에서는 구매·채용 후 ExitShop에서 노드 완료와 비용/결과를 함께 durable publish하므로 이 실제 경계에 장애를 주입한다. SavePoint 복원 역시 실제 publish에서 검사한다. IO 오류 코드21까지 확인하는 후속 oracle을 유지한다.


복원 계량 교정: 실제 Event의 장비64개 정상 지급→HQ상점 구매/완료 뒤 새프로세스 Continue에서16MiB 누적계량을 초과했다. 실패 원본을 보존하고 BTreeMap 노드 점유에 근거하여 실제 Value producer 두 곳의 과대예약만 정밀화했다. typed decode·누적3회·1byte부족 실패 원자성·기본 한도는 유지했다. RESTORE_MEMORY_REVIEW와 실제allocator3회/형태30개, 최종 default-release host 확장 완료Continue가 교정을 뒷받침한다.

검증 준비 교정: normal 초기 support-only 시나리오는 실제 전투가 없어 lab.prepare가 거절했다. 실제 지원활동을 준비 단계에서 수행한 뒤 전투를 포함했다. 확장64개를 동일37단계 준비에 넣으면 준비 누적work 한도에 닿으므로 입력이나 한도를 줄이지 않고 기존24단계 전투 기준을 별도 승인했다. 확대된 실제 상점/Continue는 별도 호스트 replay에서 검증한다. calibration의 시간은 준비를 포함한 유한 시나리오이며 단독 구매시간으로 주장하지 않는다. 첫 expanded calibration의 private root mode755는 환경 준비 오류로 보존하고700인 새 디렉터리를 사용했다.

검증 oracle 교정: 침식 variant 실제 문자열은 Corrupt이며 초기 Python oracle의 Corrupted 기대가 잘못됐다. 이미 통과한 동일 request/result는 재쓰기 없이 재사용했고 최종 소스에서는 별도 fresh stores로 전 경로를 실행했다. cost002는 paid_cost001이 아닌 초기 cost001 probe를 잘못 선택해 출처 allowlist에서 거절됐다. 입력이나 allowlist를 바꾸지 않고 원래 paid probe를 재빌드한 cost003의 동일 workload6회로 닫았다. final-host-flow001은 pilot 완료 전 호출하여 결과 파일이 없었던 orchestration 오류이며 입력 변경 없이 완료 후 successor002에서 실행했다.

상품별 ownership: allow_duplicates는 동일 직원 중복장착 계약이므로 유물 구매/보유를 제한하지 않는다. 상점은 새 사본을 정상 variant로 공통 grant하고 실제 침식은 기존 보유사본+침식촉매 명령이다. 무료 Event 선택은 별도 authority이며 현재 external public15 및 authored fresh-process44 관련 회귀가 이를 재확인한다. 구 개발 저장을 임의 재작성하여 현재 성공으로 만드는 경로는 추가하지 않았다.


소비품 승패 보강은 실제 HQ 완료 저장본을 그대로 입력으로 사용했다. 지도 좌표가 존재한다는 것과 그 좌표가 공개 배치 구역이라는 것은 다르며, 직원의 지상/플랫폼 적성도 별도로 충족해야 한다. 실패한 좌표·직원·교대 시도는 그대로 보존했다. 공개 미리보기로 확인한 지상4,4의 기존 전위와 지상4,6의 원거리 직원을 함께 배치하자18tick 승리했다. 단독 전위는25tick 패배했다. 두 결과에서 모두 후퇴 불차감·결과확정 차감·중복확인 불변을 검증하므로 승패를 가정하거나 결과를 내부에서 만들지 않는다.

최종 집계의 test-name parser는 Rust의 ` - should panic` 표시도 포함한다. 최초 집계에서 보이지 않던12건은 실행 누락이 아니었다. 정확 이름 집합2187과 source 비영향의 실제 socket1을 합쳐2188이며, 집중 실행의 중복 수를 더하지 않는다. 정식 감사는 이 증거와 범위 판단을 별도로 검토한다.
