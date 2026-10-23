# 상태·저장·메모리 확인

현재 후보/선택/등급/누적지불/합성선택은 하나의 RewardSession owner다. RelicInventory가 사본/장착/침식과 촉매를, crafting_resources가 장비 가루를 소유한다. 가루 투자 뒤 미완료 Continue는 기존 입장 전 snapshot을 택하므로 비용과 권리가 함께 되돌아간다. 최종 Claim/Confirm/Exit는 노드 완료와 결과를 기존 paired publication으로 반영한다. 별도 저장 파일이나 분리된 비용 원장이 없다.

`runs/host001`의 실제18프로세스67명령·IO6, `runs/host_edges001`의 실제 Corrupt/역순 입력2프로세스11명령, `runs/expanded_host001`의3프로세스29명령·IO2는 실파일을 다시 열어 복원했다. 각 IO_FAILURE는 OS21임을 확인했고 전후 Core roots와 active/profile bytes를 정확 비교했다. 도메인 자격 오류를 IO 실패로 세지 않았다. 실패했던 투자 중간 IO 가정은 `INVESTMENT_IO_PROBE_FAILURE.json`에 남겼으며 실패 store를 덮어쓰지 않고 그 실제 preimage에서 재개했다.

`RewardSessionState`의 추가 필드는 inline Copy이며 후보는 기존 RewardOption Vec가 소유한다. 선택 최대3개만 저장한다. 카탈로그 색인은 연산 내부 임시값이며 세션/전역 cache로 남지 않는다. 새 Reward owner의 용량과 문자열/GrantRelic ID는 relic extent에 합산하고 clone/store projection을 기존 profile에 선검사한다. 정상/확장 raw 비용은 명령과 full Run 조회를 따로 측정했다.

`COST_REVIEW.json`: 활동6프로세스312창과 공유 lifecycle6프로세스72연산 모두 끝난 뒤 allocator live requested bytes를 시작값과 비교해 정확히 회수했다. RSS는 allocator 요청량이나 native retained 회계와 같다고 해석하지 않았다. 기본 실행 native16MiB는 바꾸지 않았다. 확장300종의 별도 authoring64M 측정은 기본 admission 거절을 지우지 않는다.

구현 전 BASELINE_SOURCE_001은 실제 수정 전524파일이다. 최종 target의 BEFORE/SOURCE, SCOPED.diff, DELTA는 해당 preimage와 현재 범위만 사용한다. 복원은 소유한 /tmp 격리 사본에서 각 변경 전후 bytes를 확인한 add/replace/delete로 수행하며 정·역 방향의 전체 membership/bytes를 검사한다. 사용자 작업 tree에는 복원/reset/stash를 실행하지 않는다. 결과는 target/u16_2_current_001/RESTORE.json이다.
