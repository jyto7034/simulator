// Visual families describe a tag's nature, independently of its effect/requirement relation.
export const tagFamilies = {
  attack: { title: '공격·위험', labels: ['피해', '받는피해증가', '관통', '다중타격', '지속피해', '폭발', '투사체'] },
  healing: { title: '회복', labels: ['회복', '흡혈'] },
  defense: { title: '방어·반격', labels: ['보호막', '피해감소', '피해차단', '생존보장', '패링', '반사'] },
  control: { title: '제어·약화', labels: ['제어', '진행도', '전파'] },
  summon: { title: '소환·영역', labels: ['소환물', '방해물', '지속영역'] },
  growth: { title: '강화·축적', labels: ['기본공격강화', '능력치변경', '버프', '능력치', '스택', '스택부여', '스택획득', '스택한도', '자원', '성장', '자연감소'] },
  mechanics: { title: '시간·변형', labels: ['기본공격변형', '범위', '지속시간', '지연발동', '집중유지', '쿨다운', '주기', '시전', '대상선정'] },
  neutral: { title: '설계 상태', labels: [] },
} as const;
export type TagFamily = keyof typeof tagFamilies;
const familiesByLabel = new Map<string, TagFamily>(Object.entries(tagFamilies).flatMap(([family, value]) => value.labels.map(label => [label, family as TagFamily] as const)));
export function tagFamily(label: string): TagFamily { return familiesByLabel.get(label) || 'neutral'; }
