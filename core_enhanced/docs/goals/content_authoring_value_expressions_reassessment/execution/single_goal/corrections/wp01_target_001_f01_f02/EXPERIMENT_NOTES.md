# R06 WP01 F01/F02 교정 기술 메모

## 고정 판단

- F01은 parser admission 뒤 검증 우회가 아니라 Serde struct field 계약의 누락이다. wire/authored 입력 타입 자체에서 거부한다.
- F02의 기준은 reference set emptiness가 아니라 lexical aggregate binding을 고려한 external-frame dependency다. 기존 evaluator/narrowing을 재사용한다.
- `AggregateMember`는 authored member evaluation body 안에서만 local이다. 그 밖의 member read와 stat/resource/result 참조는 static 값으로 꾸미지 않는다.
- WP02 publication block과 runtime-unconnected 상태는 의도적으로 유지한다.

## 잔여 범위

- `WireValueExpr`의 enum-level `deny_unknown_fields`가 Mul/Ratio/Clamp/Aggregate struct variant를 덮고, `ValueRef`의 동일 설정이 Stat/HitPoints/Resource/Result/AggregateMember payload를 덮는다. 이미 strict한 AmountExpr, TypedInteger, AuthoredMember, GroupOwner, ProjectedTile, SkillEffectDef/authoring wrapper는 유지했다.
- 정적 분류는 compiled requirements를 사용한다. `ResolvedTargets`가 없고 모든 reference가 정확히 `ValueRef::AggregateMember`일 때만 external-frame-free로 본다. compiler가 member read의 aggregate lexical scope를 이미 검증하고 resolved source를 별도 flag로 기록하므로 authored member 값만 local하게 허용된다.
- `Stat`/`HitPoints`의 `AggregateMember` subject는 `AggregateMember` value read와 다른 reference이며 runtime member value map을 요구한다. source stat과 resolved-target aggregate도 dynamic 대조군으로 compile되며 빈 frame으로 평가하지 않았다.
- 정적으로 분류된 plan은 기존 `evaluate`와 `ExprMeter`를 그대로 거쳐 기존 `narrow_damage`/`narrow_heal`에 들어간다. 별도 aggregate 계산, 0/default substitution, cost/fingerprint 변경은 없다.
- source identifier는 static Damage overflow에서 source file, definition, action, effect, field까지, 외부 pack의 F01/F02에서 definition/action/effect까지 확인했다.

잔여 위험은 reviewer가 전체 RON 입력 공간을 재감사하지 않았다는 점과 WP02 runtime consumer가 여전히 미연결이라는 점이다. 후자는 의도된 경계이며 publication gate가 계속 차단한다. 이 문서는 재검토 verdict나 다음 구현 승인 문서가 아니다.
