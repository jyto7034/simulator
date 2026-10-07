# D07-B F01 현재 판단

- 원인 확정: simulate만 exact argc 검사를 건너뛰어 필수 REQUEST가 없는 3개 인자에서 args[3] panic.
- 채택: minimum positional 길이 검사를 모든 operation에 적용하고 비simulate exact 길이는 유지.
- 불필요: panic catch, parser 라이브러리 교체, 테스트를 위한 production parser 분리.
- 시험은 실제 바이너리와 structured diagnostic/exit/stdout을 확인한다. invalid path는 parse 단계 거부임을 확인하기 위한 값이다.
- 선행 실패 이력의 구형 pack 문제를 반복하지 않도록 D07-B에 동결된 현재 contract fixture를 재사용한다.
- F02/F03은 이번 CLI 수정으로 해소되지 않는다. 신규 경계 검증을 기존 runtime 시험으로 대체하지 않는다.
- 공유 worktree에서는 이 구현 대화만 제품 소스를 변경한다. 사본 검토는 successor 동결 후 진행한다.

## 교정 결과의 의미

교정 전 정확한 panic이 검출되고 같은 회귀가 교정 후 통과했다.
정상 option 조합의 amount 출력이 predecessor와 byte 동일하므로 이 parser 수정은 기존 관측 문서 계약을 바꾸지 않는다.
최종 source의 core library/data/dependency 바이트는 모두 그대로다. 해당 runtime 전체 시험과 PC36/R05는 재실행하지 않고
동결 증거의 적용성을 확인해 재사용한다. 전체 1,538개를 이번 CLI 교정 소스에서 새로 실행했다고 보고하지 않는다.
F01/C01만 자체 검증 완료 판단 대상으로 삼으며 B18 전체(큰 i128 trace 포함)나 F03 완료는 주장하지 않는다.
# 동결 결과 파일 주의

동결 후 작성되는 stdout 결과를 동결 대상에 포함하면 자기 참조가 생긴다.
첫 시도의 별도 verify가 이 문제를 검출했다. 결과 파일은 명시적으로 제외하고 직접 출력한다.
실패 후보·원본 스크립트·결과는 evidence/failed_freeze_001에 보존했다. 제품 검증 실패와 구별한다.
