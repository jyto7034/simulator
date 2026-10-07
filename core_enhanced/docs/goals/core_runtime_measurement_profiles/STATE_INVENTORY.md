
## GEX10 ACCOUNTING의 실제 관측 owner 범위

identity004 source391/438252f556ab5dafb538a2694a04c3ac4c5ede6686018b42e7e2e8b47c004114에서 Budget observation은 native scope 시작 seed와 양의 growth admission을 별도로 기록한다. legacy admitted_bytes는 거절 seed도 포함하는 attempted 역사 값이다. raw native entry의 unknown warning은 nested Core가 설정하지 않으며 actual settings-bearing 최초 entry만 nominal warning을 기록한다. owner hardcap은 RuntimeFactLimits의 실제 값이다.

CommittedSnapshot.owner_extent는 World의 기존 마지막 battle whole-owner scan과 pending_resource_extent 계산 반환값을 재사용한다. Unmeasured는 계산/게시 증거 부재, NoNativeOwner는 성공한 검사에서 battle/pending owner 모두 부재, Battle/PendingResources/Both는 각각의 이미 계량된 ledger 범위다. 0-byte owner는 Some0로 표시되어 부재와 다르다. 배틀 whole-owner scan은 기존 retained_runtime_fact_extent의 units/queue/actions/sustained/independent/runtime facts/resources 등 닫힌 범위이며, pending은 resource_runtime.rs의 confirmed facts+run resources 또는 typed result+retained facts 범위다. 둘을 heap/RSS나 서로의 bytes와 합산하지 않는다. 기존 Usage의 extent/aggregate는 battle 범위 그대로다.

max_battle_owner_extent_bytes/max_pending_owner_extent_bytes는 각각 성공한 scan/계산의 관측 최대이고 failed request는 별도 owner/cumulative requested field다. 실패/폐기 candidate는 prepared/peak이며 committed current로 게시하지 않는다. pending native operation 시작은 기존 pending_resource_extent를 seed로 사용한다. 일반 non-battle rollback capture의 heap은 native 계량 밖이며 native retained=None로 표시되고 process allocator 범위에는 포함된다. 고정 identity는 observer가 actual GameData와 run_seed를 entry/prepared/committed별로 읽으며 generation 교체와 rollback 후 owner를 따른다.
