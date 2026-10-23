import { Link } from 'react-router';
import { tagFamilies, tagFamily } from './tagPalette';
import type { ContentTag } from '../types';

export const tagRelations = { effect: '효과 태그', require: '유물 대상 태그', synergy: '유물 연결 태그' } as const;
export function TagList({ tags, category, compact = false }: { tags?: ContentTag[]; category?: string; compact?: boolean }) {
  if (!tags?.length) return null;
  const labels = [...new Set(tags.map(t => t.label))];
  const visible = compact ? labels.slice(0, 3) : labels;
  return <div className="content-tags flex flex-wrap items-start gap-1.5" aria-label="콘텐츠 태그">{visible.map(label => <Link key={label} to={'/categories/' + category + '/?tag=' + encodeURIComponent(label)} data-tag-family={tagFamily(label)} className="tag-chip max-w-full truncate rounded-md border px-2 py-0.5 text-[11.5px] leading-5" title={tags.filter(t => t.label === label).map(t => tagRelations[t.relation] + ': ' + t.note).join('\n')}>#{label}</Link>)}{compact && labels.length > visible.length && <span className="rounded px-1 py-0.5 font-mono text-[11px] leading-5 text-faint" aria-label={'태그 ' + (labels.length - visible.length) + '개 더 있음'}>+{labels.length - visible.length}</span>}</div>;
}
export function TagDetails({ tags, category }: { tags?: ContentTag[]; category?: string }) {
  if (!tags?.length) return null;
  return <div className="panel mb-7 p-4"><TagList tags={tags} category={category} /><details className="group/why mt-3 text-xs"><summary className="inline-flex items-center gap-1.5 text-muted transition-colors before:inline-block before:transition-transform before:content-['›'] hover:text-brand group-open/why:before:rotate-90">태그의 의미와 적용 근거</summary><p className="my-3 leading-6 text-muted">태그는 공통된 방식으로 바꿀 수 있는 효과 단위입니다. 유물은 대상 태그·적용 조건·변경 규칙을 갖습니다. 효과 태그와 유물이 참조하는 태그를 구분하며, 한 문서의 여러 태그가 같은 효과에 속한다는 뜻은 아닙니다. 합의·후보·미정은 근거 문서를 따릅니다.</p><ul className="space-y-3">{tags.map((t, i) => <li key={i} className="leading-6"><span className="font-semibold">{t.label}</span><span className="mx-2 rounded bg-tint px-1.5 py-0.5 text-[10.5px] text-muted">{tagRelations[t.relation]}</span>{t.note} <Link to={'/wiki/' + t.source + '/'} className="text-brand underline decoration-brand/40 underline-offset-4 hover:decoration-brand">근거 문서</Link></li>)}</ul></details></div>;
}

export function TagColorLegend() {
  return <details className="mt-3 text-[11px] text-muted"><summary className="transition-colors hover:text-brand">태그 색상 안내</summary><div className="mt-2 flex flex-wrap gap-1.5" aria-label="태그 색상 분류">{Object.entries(tagFamilies).map(([family, { title }]) => <span key={family} data-tag-family={family} className="tag-chip rounded-md border px-2 py-0.5">{title}</span>)}</div></details>;
}
