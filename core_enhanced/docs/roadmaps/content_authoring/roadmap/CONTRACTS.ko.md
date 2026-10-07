# 공유 계약 owner 표

계획상 owner와 연관 작업이다. CA01은 전체 interface 초안을 조정하지만 모든 기능을 선구현하지 않는다. 각 owner의 실제 구현에 앞서 초안을 확정할 수 있고 DG에서 실제 소비자와 정합성을 확인한다.

| K | 주요 owner | 함께 갱신할 작업 | 소비자 | 의미 |
| --- | --- | --- | --- | --- |
| K01 | CA01 | CA04, CA06, CA10, CA11 | 모든 runtime/domain | ID·source slot·binding·RNG identity |
| K02 | CA01 | CA02, CA07, CA12 | 모든 compiler·tools | typed authoring·compiled ref·descriptor·SourceMap |
| K03 | CA02 | CA19 | run/battle/cache/tools/save | generation pin·fingerprint·저장 일치 |
| K04 | CA07 | CA08, CA14, CA16 | boss/skill/basic/delivery | ActionHandle·milestone·terminal/cancel |
| K05 | CA04 | CA08, CA09, CA10, CA14 | state/resource/aura/relic/run | owner×lifetime·ResourceTxn·lease |
| K06 | CA06 | CA09, CA13, CA18 | reactive/boss/wave/result | 사건·receipts·mutation boundary |
| K07 | CA12 | CA10, CA11 | equipment/relic/tools/power | selector·patch·closure·conflict·effective plan |
| K08 | CA13 | CA06, CA15 | reactive/delayed/relation | chain frame·root/fork/join·budget |
| K09 | CA16 | CA14, CA17, CA18 | boss/fieldwork/wave/objective | typed scenario signal·binding·authority |
| K10 | CA03 | CA01, CA16, CA22 | tools/tests/all domains | diagnostic·semantic trace·receipt |

공유 계약 변경은 구현 파일만 고치지 않고 이 표·해당 architecture·CA·test matrix와 실제 project canonical에 연결한다. 한 K를 소유한다고 다른 도메인의 gameplay 정책까지 결정할 권한을 가지는 것은 아니다.

계약의 owner 표는 후보 책임 배정이며 CA 순서나 선행 작업 전체 완료를 증명하지 않는다. 실제로 소비하는 계약 부분·구현 owner·검증 시점은 [의존성 기준](DEPENDENCIES.ko.md)과 선택 범위의 Phase Plan에서 확인한다.
