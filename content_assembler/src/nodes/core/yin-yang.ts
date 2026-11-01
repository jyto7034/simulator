import { exampleModifiers } from "./example-modifiers";
import { skillExampleBuilder } from "./skill-example-builder";

export function yinYang() {
  const b = skillExampleBuilder("음양 · 음양의 순환");
  const { add, link, seconds } = b;
  b.frame("두 위치 선택 · 공명과 위치의 원자적 접수");
  const settings = b.settings();
  const range = add("range", "두 중심의 선택 가능 범위 · 미정");
  const selection = add("two-tile-selection", "음·양 순서로 두 타일 지정");
  link(range, "range", selection, "range");
  const request = add("request", "두 위치 확정 입력");
  link(selection, "selection", request, "selection");
  const use = b.admission(request, settings);
  b.frame("독립 두 영역 · 중복 회복 제외 · 반대편 전달 · 교집합 버프");
  const pattern = add("range", "음과 양의 공통 효과 패턴 · 미정");
  const duration = seconds("두 영역 공통 수명 · 미정");
  const pair = add("paired-areas", "두 영역 생성 · 생성 후 사용 완료");
  link(use, "out", pair, "in");
  link(use, "use", pair, "use");
  link(pattern, "range", pair, "first");
  link(pattern, "range", pair, "second");
  link(duration, "value", pair, "duration");
  const interval = seconds("회복 주기 · 첫 회복은 한 주기 뒤");
  const pulse = add("pair-pulse", "두 영역 대상 집합 · 같은 회차");
  link(pair, "pair", pulse, "pair");
  link(interval, "value", pulse, "interval");
  const power = add("healing-amount", "대상당 기본 회복 · 미정");
  const heal = add("pair-base-heal", "기본 회복 한 번 · 실제 결과 확정");
  link(pulse, "out", heal, "in");
  link(pulse, "pulse", heal, "pulse");
  link(power, "power", heal, "power");
  const transfer = add("pair-transfer", "HP 상한 초과분만 전달 · 재전달 없음");
  link(heal, "out", transfer, "in");
  link(heal, "result", transfer, "result");
  link(pulse, "pulse", transfer, "pulse");
  const buff = add("attack-speed-buff", "교집합 기본 공격 속도 증가 · 미정");
  const intersection = add(
    "pair-intersection",
    "교집합 진입 부여 · 이탈 즉시 해제",
  );
  link(pair, "pair", intersection, "pair");
  link(buff, "buff", intersection, "buff");
  const cue = add("cue", "음·양 영역과 버프 표시", {
    description:
      "음과 양의 고정 영역을 구분해 표시합니다. 교집합 기여분은 진입·이탈·교체·종료에 맞춰 표시하며 연출로 피해나 추가 회복을 만들지 않습니다.",
  });
  for (const n of [pair, heal, transfer, intersection])
    link(n, "cue", cue, "event");
  b.frame("장착 유물 → 두 영역 공통 정의 변경 · 선택 거리는 유지");
  const m = exampleModifiers(b);
  m.range(pattern);
  m.duration(duration);
  m.bind(
    "healing-increase",
    "온기의 성배 · 기본 회복 요청량",
    "healing",
    power,
    "power",
  );
  m.bind(
    "vivid-blessing",
    "선명한 축복 · 교집합 버프 수치",
    "buff",
    buff,
    "buff",
  );
  m.healingBuff(power);
  const burst = m.bind(
    "yin-yang-dragon",
    "승천의 징표 · 같은 중심 일회 판정",
    "coincident-area-burst",
    pair,
    "pair",
    { percent: 25 },
  );
  link(
    add("range", "승천의 징표 · 일회 영역 타일 미정"),
    "range",
    burst,
    "range",
  );
  link(seconds("승천의 징표 · 출현 판정 지연 미정"), "value", burst, "delay");
  return b.finish();
}
