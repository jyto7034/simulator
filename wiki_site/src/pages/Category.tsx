import { useEffect, useState } from 'react';
import { Link, useParams, useSearchParams } from 'react-router';
import { categoryPages, useData } from '../data';
import { Breadcrumb, Empty, IconTile } from '../components/common';
import { Search } from 'lucide-react';
import { Collection, documentEntry } from '../components/Collection';
import { tagFamily } from '../components/tagPalette';
import { tagRelations, TagColorLegend } from '../components/Tags';

export function CategoryPage({archive=false}:{archive?:boolean}) {
  const { id } = useParams();
  return <CategoryContent key={archive?'history':id} id={archive?'history':id} />;
}
function CategoryContent({ id }: { id?: string }) {
  const { meta } = useData();
  const [query,setQuery]=useState(''),[tier,setTier]=useState('');
  const [params, setParams] = useSearchParams();
  // Static routes are rendered without a query string. Apply URL filters after hydration.
  const [ready, setReady] = useState(false);
  useEffect(() => setReady(true), []);
  const selected=ready?[...new Set(params.getAll('tag'))]:[], mode=ready&&params.get('match')==='any'?'any':'all';
  const relation=ready?params.get('relation')||'':'';
  const category=meta.categories.find(c=>c.id===id);
  if(!category)return <Empty>분류를 찾을 수 없습니다.</Empty>;
  const relics=id==='relics',tagged=relics||id==='fragments',pages=categoryPages(meta.pages,category.id),tiers=relics?['범용','전용','구분 미정']:[...new Set(pages.map(p=>p.tier).filter(Boolean))];
  const entries=pages.map(p=>documentEntry(p,meta));
  const terms=query.trim().toLowerCase().split(/\s+/).filter(Boolean);
  const candidates=entries.filter(p=>terms.every(t=>[p.title,p.summary,p.kind,...(p.tags||[]).map(t=>t.label),...(p.facts||[]).map(f=>f.label+' '+f.value)].join(' ').toLowerCase().includes(t))&&(!tier||p.kind.startsWith(tier)));
  const tagLabels=(p:typeof entries[number])=>[...new Set((p.tags||[]).filter(t=>!relation||t.relation===relation).map(t=>t.label))];
  const found=candidates.filter(p=>!selected.length||(mode==='all'?selected.every(t=>tagLabels(p).includes(t)):selected.some(t=>tagLabels(p).includes(t))));
  const available=[...new Set([...entries.flatMap(tagLabels),...selected])].sort((a,b)=>a.localeCompare(b,'ko'));
  function toggleTag(label:string) {
    const next=new URLSearchParams(params);next.delete('tag');
    (selected.includes(label)?selected.filter(t=>t!==label):[...selected,label]).forEach(t=>next.append('tag',t));
    setParams(next,{preventScrollReset:true});
  }
  function changeFilter(key:string,value:string) {const next=new URLSearchParams(params);if(value)next.set(key,value);else next.delete(key);setParams(next,{preventScrollReset:true});}
  function resetTags() {const next=new URLSearchParams(params);['tag','match','relation'].forEach(k=>next.delete(k));setParams(next,{preventScrollReset:true});}
  return <><Breadcrumb items={[{title:category.title}]} /><div className="flex items-center gap-5"><span className="hidden sm:block"><IconTile category={category.id} size="lg" /></span><div className="min-w-0 flex-1"><p className="eyebrow">ENCYCLOPEDIA / {id?.toUpperCase()}</p><h1 className="page-title mt-2">{category.title}</h1></div><span className="hidden shrink-0 text-right sm:block"><b className="block font-serif text-[30px] leading-none font-bold tabular-nums">{pages.length}</b><span className="kicker">항목</span></span></div><p className="page-lead">{category.description}. 요약을 비교하고, 빠르게 보기에서 목록을 떠나지 않고 상세 내용을 읽어보세요.</p>
    {id==='history'&&<p className="notice mb-6">현재 규칙으로 사용하지 않는 보존 자료입니다. <Link className="text-brand underline" to="/wiki/archive/search/">아카이브 본문 검색</Link> · <Link className="text-brand underline" to="/">현재 위키로 돌아가기</Link></p>}
    <div className="mb-4 flex flex-wrap gap-3"><div className="relative min-w-0 flex-1"><Search className="pointer-events-none absolute top-1/2 left-3.5 size-4 -translate-y-1/2 text-faint" aria-hidden="true" /><input className="field w-full min-w-0 pl-10" type="search" aria-label={relics?'유물 검색':'목록 검색'} placeholder="이름·내용·조건·태그 검색…" value={query} onChange={e=>setQuery(e.target.value)} /></div>{tiers.length>0&&<select className="field" aria-label={relics?'유물 구분':'분류 필터'} value={tier} onChange={e=>setTier(e.target.value)}><option value="">모든 분류</option>{tiers.map(t=><option key={t}>{t}</option>)}</select>}</div>
    {tagged&&<section aria-label="태그 필터" className="panel mb-6 p-4 md:p-5">
      <div className="flex flex-wrap items-center gap-3"><h2 className="mr-auto flex items-center gap-2 text-sm font-semibold"><span className="font-mono text-brass">#</span>태그로 찾기{selected.length>0&&<span className="rounded-full bg-brand px-2 py-px text-[11px] font-medium text-paper tabular-nums">{selected.length}</span>}</h2><button type="button" onClick={resetTags} className="rounded-md px-2 py-1 text-xs font-medium text-brand transition-colors hover:bg-tint disabled:text-faint disabled:hover:bg-transparent" disabled={!selected.length&&!relation&&mode==='all'}>태그 초기화</button></div>
      <div className="my-3 flex flex-wrap gap-2"><select className="field min-w-0 max-w-full text-xs" aria-label="태그 조합 방식" value={mode} onChange={e=>changeFilter('match',e.target.value==='all'?'':e.target.value)}><option value="all">선택한 태그 모두 포함</option><option value="any">선택한 태그 하나 이상 포함</option></select><select className="field min-w-0 max-w-full text-xs" aria-label="태그 관계" value={relation} onChange={e=>changeFilter('relation',e.target.value)}><option value="">모든 태그 관계</option>{Object.entries(tagRelations).map(([value,label])=><option key={value} value={value}>{label}</option>)}</select></div>
      <div className="flex max-h-52 flex-wrap gap-2 overflow-y-auto p-1" role="group" aria-label="태그 선택">{available.map(label=>{const count=candidates.filter(p=>tagLabels(p).includes(label)).length;return <button type="button" key={label} aria-pressed={selected.includes(label)} aria-label={'태그: '+label} onClick={()=>toggleTag(label)} data-tag-family={tagFamily(label)} className="tag-chip rounded-md border px-2.5 py-1 text-xs"><span aria-hidden="true" className="inline-block w-3.5 opacity-70">{selected.includes(label)?'✓':'#'}</span>{label}<span className="ml-1.5 font-mono text-[10.5px] tabular-nums opacity-70">{count}</span></button>;})}</div>
      <TagColorLegend />
      {selected.length>0&&<p className="mt-3 text-xs leading-6" aria-label="선택한 태그">선택: {selected.join(' · ')} <span className="text-muted">({mode==='all'?'모두 포함':'하나 이상'})</span></p>}
      <p className="mt-3 text-[11px] leading-5 text-muted">숫자는 검색·분류·태그 관계에 맞는 항목 수입니다. 태그는 변경 가능한 효과의 분류이며 유물 이름과 구분합니다. 효과·유물 대상·유물 연결 관계를 나누며, 한 문서의 여러 태그가 같은 효과에 속한다는 뜻은 아닙니다. <Link to="/wiki/content/tags/README/" className="text-brand underline">태그 사전</Link></p>
    </section>}
    <p className="mb-3 text-xs text-muted" role="status"><b className="font-semibold text-ink tabular-nums">{found.length}</b> / {pages.length}개 항목</p>
    <Collection entries={found} scope={'category:'+category.id} label={category.title} profiles={['abnormalities', 'fragments', 'equipment', 'relics'].includes(category.id)} />
  </>;
}
