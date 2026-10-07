# R13~R16 조사와 계획 준비

2026-10-02. **R08~R12 선행 독립 감사 대기. 제품 구현 미착수.**

사용자는 선행이 충족되면 정식 계획→R13→R16→최종 자체 검증·동결까지 연속 수행하고, 선행 감사가 없으면 조사·계획 준비와 잔여 조건 보고를 요청했다. [상세 지시서](../../../../roadmaps/content_authoring/handoffs/R13_R16_CONTINUOUS_IMPLEMENTATION_REQUEST_20261002.ko.md) §2는 이번 요청만으로 선행 GEX-20 감사를 자동 실행하지 않도록 명시한다. 내부 WP별 검토 대기와 정책 승인 대기는 선택하지 않았다.

## 선행 인수 결과

- [현재성 확인](CURRENTNESS.json): 동결 manifest SHA-256 `9842985297f11fefcfeda91cade469cee4f172a74f4498238a02e7e758686a1c`는 FINAL_TARGET 포인터와 일치한다. frozen source는 `bd89f9c42191217a333d648d35f495e802d13e6fb1420caa9c08da3deb731690`, 387개다.
- 현재 membership은 같은 387개이며 추가·누락은 없다. 386개는 동일하고 `ROADMAP_STATUS.md`만 변경되어 현재 digest는 `a1289812bb0083a419c0767a5f157317e67e4c7602c368b230f5193f455eec8b`다. 따라서 전체 source가 exact 동일하다고 보고하지 않는다. 제품 코드 차이는 발견되지 않았다.
- frozen references와 현재 문서는 `equipment-and-fragments.md`, `open-decisions.md`, `ROADMAP_STATUS.md`가 다르다. [참조 차이](REFERENCE_DELTA.json)와 patch는 동결 당시 bytes에서 읽은 별도 자료다. 최신 유물 합의를 과거 R08~R12 감사 basis에 소급 적용하지 않는다.
- manifest가 가리키는 archive/patch 9개 hash는 일치한다. 이것은 보존 확인이며 독립 동작 감사가 아니다.
- [감사 탐색](AUDIT_DISCOVERY.json)의 후보를 확인했다. R06/R07의 Complete / High는 source `71419532…365e6f`에 대한 판정이다. R08~R12의 FINAL_TARGET은 `independent_audit: false`; 후속 R08~R12 완료 판정은 확인되지 않았다. 저장소 밖의 감사 존재 여부는 단정하지 않는다.
- R08~R12 인계는 55조건 중54 자체 검증·X09 환경 제한1이다. Unix socket fixture 제외 후 각1676 통과는 원명령 전체 PASS가 아니다. 이번에 제품 시험을 재실행하지 않았다.

## 준비 산출물

| 자료 | 역할과 한계 |
| --- | --- |
| [코드 조사와 구현 설계 초안](DESIGN_PREPARATION.ko.md) | 공개 입력부터 결과/저장/조회까지 경계, WP13~16, AT13~15, 후속 세부 설계 항목 |
| [수용표 초안](ACCEPTANCE_DRAFT.json) | 지시서25조건과 기존 T 조건, 공개 입력·oracle·후보 selector. 모두 미실행 |
| [정책 판단](EXPERIMENT_NOTES.md) | 사용자 직접 합의와 구현자 위임 결정을 구분. 승인 질문을 남기지 않음 |
| [정식 계획 준비](P2G_PREPARATION.ko.md) | Single Goal 후보, 필요한 artifact/validator와 선행 충족 뒤 수행 순서 |
| [선행 감사 인계](PREREQUISITE_AUDIT_REQUEST.ko.md) | exact GEX-20 대상, X09, 공유 경계·원자료·문서 delta 처리 |
| [작업 기록](EXPERIMENTS.md) | 실제 실행·미실행, 보존과 준비 검사 |
| [원본 목록](BEFORE_MANIFEST.json) | 기존 dirty/untracked 실제 bytes를 SOURCE_BEFORE.tar.gz에 보존; git worktree/index patch 별도 |
| [준비 검사](PREPARATION_CHECK.json) | 문서 링크·25조건 대응·기존 자료 불변·보존 archive 격리 복원. 게임/P2G/감사 PASS가 아님 |

## 선행이 충족된 뒤 재개 순서

1. R08~R12 GEX-20의 exact target·판정·차단 finding·X09 결과를 인수한다. 교정 source가 생겼다면 새 currentness와 영향 조사로 이번 초안을 갱신한다.
2. 준비 자료를 재사용하여 P2G-00/10/20→30→40 receipt 생성·재검증→50 quality2→60을 각각 수행한다. 기존 카드를 Goal로 간주하지 않는다.
3. 새 `execution/single_goal/`에서 baseline을 다시 보존하고 GEX-00/10을 수행한다. WP13→14→15→16→통합 검증이며 내부 WP마다 승인을 요구하지 않는다.
4. 공개 RON·정상 Core·전투·저장/fresh Core, default/feature 전체 시험·CLI·PC36/R05, source 현재성과 격리 정/역 복원을 검증한다. 기존 runner의 고정 출력은 재사용하지 않는다.
5. 필수 조건이 모두 연결된 최종 통합 target과 인계서를 동결한다. 구현자는 독립 PASS를 발행하지 않고 독립 감사 요청 대기로 종료한다.

이번 결과는 **조사·계획 준비 완료 / 선행 감사 대기**다. 정식 계획 검증, WP13~16 구현, 모든 신규 기능 시험, 최종 구현 target 동결은 미완료다. R17+, 차기 원정 전체, GUI/서버 확장, stress/metrics/20분 시험은 범위 밖이다.
