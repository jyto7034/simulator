# C03 현재 판단

Damage0은 command를 생략하지만 관측 record는 평가 사실을 보관한다. Heal0은 command/완료 fact를 남기되 HP 변화는0이다. 음수 Heal은 부호를 유지하여 HP를 감소시킨다.
Damage i32::MIN의 amount는2147483648u32이며 Heal i32::MIN은-2147483648i32다. HP 결과만으로 narrowing을 판정하지 않는다. Damage 후속 formula의 raw 값과 command base_amount도 구별한다.
새 record와 두 encoder 재평가를 추가로 확인하되 public selector/terminal/한도/취소는 C06–C15 잔여로 유지한다.

수정된 준비 경계: C03은 runtime/catalog legacy 호환 경계이며 음수 Damage 공개 허용이 아니다. 기존 validate_contract/set_effect_amount의 음수 Damage 거부를 먼저 확인한다. 음수 Damage 전용 cfg(test) staging은 real compile_skills를 사용하고 정상 공개 경로를 우회했다고 명시한다. Heal MIN은 기존 typed authoring API로 준비하여 정상 validation/compiler를 거친다. 원문 RON MIN parse 문제는 이번 C03의 loader 개선으로 확대하지 않으며 실패 원자료와 인계에 남긴다.
