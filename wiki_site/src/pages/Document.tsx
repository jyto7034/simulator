import { useEffect, useState } from 'react';
import { Link, Navigate, useLocation } from 'react-router';
import { ArrowUpRight, ExternalLink } from 'lucide-react';
import clsx from 'clsx';
import { Breadcrumb, Empty, IconTile } from '../components/common';
import { TagDetails } from '../components/Tags';
import { Markdown } from '../components/Markdown';
import { partNames, pathFor, useData, usePage } from '../data';
import type { Heading } from '../types';

/** Tracks the last heading scrolled past so the table of contents can mark the current section. */
function useActiveHeading(toc: Heading[] | undefined) {
  const [active, setActive] = useState('');
  useEffect(() => {
    if (!toc?.length) return;
    const elements = toc.map(h => document.getElementById(h.anchor)).filter((e): e is HTMLElement => !!e);
    let frame = 0;
    function update() {
      frame = 0;
      let current = '';
      for (const e of elements) { if (e.getBoundingClientRect().top < 140) current = e.id; else break; }
      setActive(current);
    }
    const schedule = () => { if (!frame) frame = requestAnimationFrame(update); };
    update();
    window.addEventListener('scroll', schedule, { passive: true });
    return () => { window.removeEventListener('scroll', schedule); cancelAnimationFrame(frame); };
  }, [toc]);
  return active;
}
function ReadingProgress() {
  const [progress, setProgress] = useState(0);
  useEffect(() => {
    let frame = 0;
    const update = () => { frame = 0; const max = document.documentElement.scrollHeight - innerHeight; setProgress(max > 0 ? Math.min(1, scrollY / max) : 0); };
    const schedule = () => { if (!frame) frame = requestAnimationFrame(update); };
    update(); window.addEventListener('scroll', schedule, { passive: true }); window.addEventListener('resize', schedule);
    return () => { window.removeEventListener('scroll', schedule); window.removeEventListener('resize', schedule); cancelAnimationFrame(frame); };
  }, []);
  return <div className="pointer-events-none fixed inset-x-0 top-0 z-30 h-[2px] lg:left-64" aria-hidden="true"><div className="h-full origin-left bg-brass" style={{ transform: `scaleX(${progress})` }} /></div>;
}

export function DocumentPage() {
  const location=useLocation(),{meta}=useData();
  const pathname=location.pathname.replace(/\/?$/, '/');
  const metadata=meta.pages.find(p=>p.path===pathname||p.legacyPath===pathname),id=metadata?.id||'';
  const {page,error}=usePage(id);
  const active=useActiveHeading(page?.toc);
  if(!metadata)return <Empty>문서를 찾을 수 없습니다. <Link to="/" className="text-brand underline">위키 홈</Link></Empty>;
  if(metadata.legacyPath===pathname)return <Navigate replace to={metadata.path+location.search+location.hash}/>;
  const category=meta.categories.find(c=>c.id===metadata.category)!;
  const order=['README','encounter','fragment','equipment','synergies','reference','review'];
  const related=metadata.entity?meta.pages.filter(p=>p.entity===metadata.entity&&!p.archived).sort((a,b)=>order.indexOf(a.part||'')-order.indexOf(b.part||'')):[];
  const toc=page?.toc.filter(h=>h.level<=3)||[];
  const plays=['fragments','equipment'].includes(metadata.category)?[['/scenes/loadout/','직원 로드아웃'],['/scenes/maintenance/','강화·개화·분쇄'],['/scenes/battle/','전투와 스킬 사용']]:[['/scenes/','장면·플레이어 행동'],[pathFor('rules/index'),'현재 게임 규칙']];
  return <><ReadingProgress />
    <Breadcrumb items={[{title:category.title,to:metadata.archived?'/wiki/archive/':'/categories/'+category.id+'/'},{title:metadata.title}]} />
    <header className="flex items-start gap-5"><span className="mt-1 hidden sm:block"><IconTile category={category.id} size="lg" /></span><div className="min-w-0 flex-1">
      <p className="eyebrow eyebrow-ko">{category.title}{metadata.tier?' / '+metadata.tier:''}</p><h1 className="page-title mt-2">{metadata.title}</h1>
      <div className="mt-3 flex flex-wrap items-center gap-x-4 gap-y-2 text-xs text-muted"><span>{metadata.archived?'아카이브 · 현재 규칙 아님':'최신 합의 · 미정 사항은 별도 표시'}</span><a className="inline-flex items-center gap-1 font-medium text-brand hover:underline" href={metadata.source}>원문 열기<ExternalLink className="size-3" /></a></div>
    </div></header>
    <div className="mt-7">{metadata.classifications?.length ? <p className="mb-3 text-xs text-muted">문서 분류: {metadata.classifications.join(' · ')}</p> : null}<TagDetails tags={metadata.tags} category={metadata.category} /></div>
    {related.length>0&&<nav aria-label="관련 콘텐츠" className="mb-8 -mx-1 flex gap-1 overflow-x-auto border-b border-line px-1">{related.map(p=><Link key={p.id} to={p.path} aria-current={p.id===id?'page':undefined} className={clsx('relative shrink-0 px-3.5 py-3 text-[13px] transition-colors',p.id===id?'font-semibold text-ink after:absolute after:inset-x-2 after:-bottom-px after:h-[2px] after:rounded-full after:bg-brass':'text-muted hover:text-ink')}>{partNames[p.part||'']||p.title}</Link>)}</nav>}
    {metadata.archived&&<div className="notice mb-6">이전 합의·검토 제안을 보존한 아카이브입니다. 현재 규칙으로 사용하지 않습니다. {metadata.currentPath&&<Link className="text-brand underline" to={metadata.currentPath}>최신 합의 보기</Link>}</div>}
    <div className="flex flex-col gap-6 xl:grid xl:grid-cols-[minmax(0,1fr)_236px] xl:gap-12">
      <div className="order-2 min-w-0 xl:order-1">{error?<Empty>{error} <button onClick={()=>window.location.reload()} className="text-brand underline">다시 시도</button></Empty>:page?<Markdown html={page.html}/>:<div role="status" className="py-12 text-sm text-muted">문서를 불러오는 중입니다…</div>}</div>
      <aside className="order-1 text-xs xl:sticky xl:top-24 xl:order-2 xl:max-h-[calc(100dvh-7rem)] xl:self-start xl:overflow-y-auto">
        <details open className="panel p-4 xl:border-0 xl:bg-transparent xl:p-0 xl:shadow-none"><summary className="kicker cursor-pointer">이 문서에서</summary><nav aria-label="문서 목차" className="mt-3 max-h-48 overflow-y-auto border-l border-line xl:max-h-none">{toc.map(h=><Link key={h.anchor} to={metadata.path+'#'+encodeURIComponent(h.anchor)} aria-current={active===h.anchor?'true':undefined} className={clsx('toc-link',h.level===3&&'pl-6 text-[11.5px]')}>{h.title}</Link>)}</nav></details>
        <div className="panel mt-8 hidden overflow-hidden xl:block"><h2 className="kicker border-b border-line px-4 py-3">문서 정보</h2><dl className="space-y-4 px-4 py-4"><div><dt className="text-[11px] text-faint">분류</dt><dd className="mt-1 font-medium">{metadata.tier||category.title}</dd></div><div><dt className="text-[11px] text-faint">상태 읽기</dt><dd className="mt-1 leading-relaxed text-muted">{metadata.archived?'보존된 이전 기록':'최신 합의와 남은 미정'}<br/>구현 여부와 별개</dd></div><div><dt className="text-[11px] text-faint">관련 플레이</dt><dd className="mt-2 flex flex-col gap-1.5">{plays.map(([to,t])=><Link key={to} to={to} className="group inline-flex items-center justify-between gap-2 rounded-md px-2 py-1.5 -mx-2 text-brand transition-colors hover:bg-tint">{t}<ArrowUpRight className="size-3 opacity-0 transition-opacity group-hover:opacity-100" /></Link>)}</dd></div></dl></div>
      </aside>
    </div>
  </>;
}
