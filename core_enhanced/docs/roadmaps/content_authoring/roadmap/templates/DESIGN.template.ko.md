# <CAxx> 코드 기반 상세 설계

이 파일은 템플릿이다. 미확인은 미확인으로, 미실행은 미실행으로 둔다.

## 실행 맥락
선택 작업 / 실행 권한 / revision·관련 dirty / 선행 계약 revision과 evidence / 현재 읽은 범위.

## 현재 경로와 책임
입력→검증→실행→결과·저장·소비자. 실제 파일·심볼과 현재 테스트를 연결한다. 읽지 않은 파일을 확인했다고 쓰지 않는다.

## 권장안 대조
| R/K 참조 | 채택·수정·기각 | 코드 근거 | 대안 및 trade-off | 영향 소비자 |
| --- | --- | --- | --- | --- |

## 상세 구조
정확한 타입·ID·필드·포트 입력/출력·오류 / authoring→compiled→runtime shape / 데이터 원본과 runtime binding / result 소비 방식.

## 상태·순서
owner/lifetime별 생성·변경·종료 / 같은 시각의 순서 / 비용·RNG·ID·queue·log의 atomicity / cancel·stale·death·withdraw·retry·retreat·save 영향.

## 변경 계획
재사용할 심볼 / 새 타입·서비스 / 교체·삭제 경로 / schema·최소 RON·저장 전환 / descriptor·diagnostic / 외부 비범위.

## 검증 matrix
| 입력/경계 | 기대 결과·불변식 | 실행 경로 | test ID | 정책 근거 |
| --- | --- | --- | --- | --- |

## 미정과 리스크
PG·실제 blocker / 검토가 필요한 공유 계약 / 실행하지 못한 실험 / 성능 가정 / 완료 주장 범위.

## 의존성과 독립 중간 상태

기존 Code Review·Phase Plan의 해당 edge/section을 참조한다. 없으면 후보임을 표시하고 올바른 owner에 연결한다. 추가 별도 registry는 필요 없다.

| 선행 결과 → 후행 소비 지점 | 필수/동시 전환/검증/권장 관계 | 실제 source·계약 근거 | 무시할 때 깨지는 상태·완료 주장 | 유지·분리·통합·대체와 이유 |
| --- | --- | --- | --- | --- |

선택 Phase 종료 뒤 다음 Phase 없이도 loader·실행·실패/복원·저장의 해당 계약이 성립하는지 설명한다. 현재 Phase Plan과 다르면 문서 교정만으로 구현 순서를 바꾸지 않는다. 설계상 선행과 실제 선행 구현·감사 evidence를 구분한다.
