# U16-2 현재 판단

- 세 기본 유물 활동의 작성/명령/조회/공통지급/완료/저장을 구현하고 자체 검증했다. 미래 유형2개와 공식 유물 효과 확장은 미지원·후속 범위다. 독립 감사는 수행하지 않았다.
- 초기 누적 가루3/9/18(차액3/6/9)과 대표 weight1은 CF-D06 위임 튜닝이다. 후보의 rarity common/rare/special은 이 공급의 계약이며 기존 모든 유물에 전역 제한을 추가하지 않는다.
- 정상 후보 부족/빈 공급은 작성 오류가 아니다. 유료 연산은 확률/등급별 유효 공급을 사전 확인하고 기존 권리를 보존한다. 같은 종류 두 사본은 허용하며 allow_duplicates는 장착 제한이므로 획득 제한으로 사용하지 않는다.
- 준비 시나리오의 quota와 Trial 준비 종료(InBattle) 조건을 실제 입력/공개 pilot로 교정했다. 최종 calibration은 기존 SelectedEvent 포트로 필요한 동일 필드를 조회한다. 연산·assertion·battle workload를 보존하고 무관한 full Run 출력을 줄였다. 실제 full Run 비용은 별도로 측정했다.
- inventory는0인 가루 스택을 생략한다. public RelicState에는 남은 가루가 포함된다. 진행기의0스택/미이용 종료 기대값을 계약에 맞게 정정했고 원시 오류는 보존했다.
- 중간 투자 자체가 항상 파일을 발행한다는 가정은 잘못이다. 실제 저장 경계는 최종 수령/완료이며 그곳에서 IO 원자성/재시도를 검증했다. 제품의 미완료 입장 전 복원 규칙을 바꾸지 않았다.
- 정상12후보/3초기 사본은 기본 authoring/runtime profile에서 승인·실행된다. 확장300후보/67초기 사본은 기본 authoring10M GeneratedSetup에서 거절된다. 입력은 유지하고 별도 유한64M authoring 측정 profile로 전체 비용/approved host를 관측했다. 제품 기본값과native16MiB는 유지한다. 한도 자동 상향·무제한 실행·stress pack의 기본 승인 주장은 없다.
- 원시 회귀·실제host·작성오류·정확 회수·공유 before/after 비교는 TEST_RESULTS_INDEX/COST_REVIEW/BASELINE_REUSE가 연결한다. 구동 binaries 이후 변경은 cfg(test) manifest locator와 문서/진행기뿐이며 exact preimage와 비영향 근거를 남겼다.
- 다음 행동은 복원 가능한 final target 고정 후 별도 요청의 GEX-20이다. 사용자 새 요청 없이 독립 감사나 U16-3에 들어가지 않는다. 열린 구현 결함·미결 정책·필수 자체검증 누락은 없다. Complete verdict는 감사자의 책임이다.
