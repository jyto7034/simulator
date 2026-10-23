import { useEffect, useRef, useState, type ReactNode } from 'react';
import { Link } from 'react-router';
import { Dialog } from 'radix-ui';
import { ArrowRight, ArrowUpRight, BookOpen, ChevronRight, Eye, LayoutGrid, List, Table2, X } from 'lucide-react';
import clsx from 'clsx';
import { Empty, IconTile } from './common';
import { Markdown } from './Markdown';
import { usePage } from '../data';
import { TagList, TagDetails } from './Tags';
import type { ContentTag, PageMeta, SiteData } from '../types';

export interface CollectionEntry {
  id: string;
  title: string;
  kind: string;
  summary: string;
  category?: string;
  tags?: ContentTag[];
  facts?: { label: string; value: string }[];
  status?: string;
  href?: string;
  pageId?: string;
  image?: { url: string; caption: string };
  detail?: ReactNode;
  anchor?: string;
  className?: string;
}
export function documentEntry(page: PageMeta, meta: SiteData, excerpt?: string): CollectionEntry {
  const preview = page.relic || page.preview;
  return {
    id: page.id, title: page.title, kind: preview?.kind || page.tier || meta.categories.find(c => c.id === page.category)?.title || '',
    summary: excerpt || preview?.summary || page.description, category: page.category,
    tags: page.tags, facts: preview?.facts, status: preview?.status, href: page.path, pageId: page.id,
  };
}
const views = [{ id: 'cards', title: '카드', icon: LayoutGrid }, { id: 'table', title: '비교표', icon: Table2 }, { id: 'list', title: '목록', icon: List }] as const;
type View = typeof views[number]['id'];
function Facts({ entry }: { entry: CollectionEntry }) {
  return <dl className="space-y-2.5 text-[12.5px]">{entry.facts?.map((f, i) => <div key={i} className="grid grid-cols-[5.5rem_minmax(0,1fr)] gap-3"><dt className="text-muted">{f.label}</dt><dd className="leading-relaxed [overflow-wrap:anywhere]">{f.value}</dd></div>)}</dl>;
}
function Status({ entry }: { entry: CollectionEntry }) {
  return entry.status ? <details className="group/status mt-4 text-xs"><summary className="inline-flex items-center gap-1 text-muted transition-colors hover:text-brand"><ChevronRight className="size-3.5 transition-transform group-open/status:rotate-90" aria-hidden="true" />합의·미정 확인</summary><p className="mt-2 rounded-md bg-tint px-3 py-2 leading-6 text-muted">{entry.status}</p></details> : null;
}
function DocumentPreview({ id }: { id: string }) {
  const { page, error } = usePage(id);
  if (error) return <Empty>{error}</Empty>;
  return page ? <Markdown html={page.html} /> : <div role="status" className="space-y-3 py-2" aria-label="상세 내용을 불러오는 중입니다">{[92, 100, 76, 88, 60].map((w, i) => <div key={i} className="h-3.5 animate-pulse rounded bg-tint" style={{ width: w + '%' }} />)}</div>;
}

export function Collection({ entries, label, scope, initialView = 'cards', highlighted, profiles = false }: {
  entries: CollectionEntry[]; label: string; scope: string; initialView?: View; highlighted?: string; profiles?: boolean;
}) {
  const [view, setView] = useState<View>(initialView);
  const [selected, setSelected] = useState<CollectionEntry | null>(null);
  const opener = useRef<HTMLButtonElement | null>(null);
  useEffect(() => {
    try {
      const saved = localStorage.getItem('ark-wiki-view:' + scope) || (scope === 'category:relics' ? localStorage.getItem('ark-wiki-relic-view') : null);
      if (views.some(v => v.id === saved)) setView(saved as View);
    } catch { /* Keep the default when storage is unavailable. */ }
  }, [scope]);
  useEffect(() => {
    if (!highlighted) return;
    const frame=requestAnimationFrame(()=>document.getElementById(decodeURIComponent(highlighted.slice(1)))?.scrollIntoView({block:'center'}));
    return ()=>cancelAnimationFrame(frame);
  }, [highlighted, view]);
  function choose(next: View) {
    setView(next);
    try { localStorage.setItem('ark-wiki-view:' + scope, next); } catch { /* Session selection remains usable. */ }
  }
  function actions(entry: CollectionEntry, card = false) {
    return <div className={clsx('collection-actions flex shrink-0 items-center gap-3 text-xs', card && profiles ? 'pt-2.5' : 'pt-4', card && 'border-t border-line')}>
      <button type="button" className={card ? 'collection-preview' : 'inline-flex shrink-0 items-center gap-1.5 rounded-md border border-field-line bg-surface px-3 py-1.5 font-medium text-brand transition-colors hover:border-brand hover:bg-tint'} onClick={e => { opener.current = e.currentTarget; setSelected(entry); }} aria-label={entry.title + ' 빠르게 보기'} aria-haspopup="dialog">{!card && <><BookOpen className="size-3.5" />빠르게 보기</>}</button>
      {entry.href && <Link className="inline-flex items-center gap-1 font-medium text-muted transition-colors hover:text-brand" to={entry.href}>상세 보기<ArrowUpRight className="size-3.5 shrink-0" /></Link>}
      {card && <span className="pointer-events-none ml-auto inline-flex items-center gap-1.5 text-faint opacity-0 transition-opacity group-hover/card:opacity-100 group-has-[:focus-visible]/card:opacity-100" aria-hidden="true"><Eye className="size-3.5" />미리보기</span>}
    </div>;
  }
  function title(entry: CollectionEntry) {
    return entry.href ? <Link className="transition-colors hover:text-brand" to={entry.href}>{entry.title}</Link> : entry.title;
  }
  function highlight(entry: CollectionEntry) { return entry.anchor && highlighted === '#' + entry.anchor; }
  const ring = 'highlight ring-2 ring-brass ring-inset';
  return <>
    <div className="mb-5 flex flex-wrap items-center justify-between gap-3">
      <div className="segmented" role="group" aria-label={label + ' 보기 방식'}>{views.map(v => <button type="button" key={v.id} aria-pressed={view === v.id} onClick={() => choose(v.id)}><v.icon className="size-3.5" />{v.title}</button>)}</div>
      <p className="text-xs text-muted">{view === 'cards' ? (profiles ? '프로필을 누르면 전체 내용을 빠르게 볼 수 있습니다' : '카드를 누르면 전체 내용을 빠르게 볼 수 있습니다') : view === 'table' ? '내용과 조건을 나란히 비교' : '필요한 항목만 펼쳐서 읽기'}</p>
    </div>
    {!entries.length ? <Empty>일치하는 항목이 없습니다. 다른 단어를 입력하거나 필터를 바꿔 보세요.</Empty> : view === 'cards' ?
      <div className={clsx('collection-grid grid grid-cols-1 md:grid-cols-2 2xl:grid-cols-3', profiles ? 'gap-3' : 'gap-4')}>{entries.map(entry => profiles ? <article id={entry.anchor} key={entry.id} className={clsx('collection-card collection-profile group/card flex min-w-0 scroll-mt-24 flex-col rounded-xl border border-(--collection-border) bg-surface p-4', 'accent-' + (entry.category || 'none'), entry.className, highlight(entry) && ring)}>
        <header className="flex items-center gap-3">
          <IconTile category={entry.category || ''} />
          <div className="min-w-0 flex-1">
            <p className="truncate text-[11px] leading-5 text-muted" title={entry.kind}>{entry.kind}</p>
            <h2 className="card-title line-clamp-2 font-serif text-[17px] leading-6 font-bold transition-colors [overflow-wrap:anywhere]" title={entry.title}>{entry.title}</h2>
          </div>
        </header>
        <p className="mt-3 line-clamp-3 text-[13px] leading-[22px] text-ink/85 [overflow-wrap:anywhere]">{entry.summary}</p>
        {entry.tags?.length ? <div className="mt-2.5"><TagList tags={entry.tags} category={entry.category} compact /></div>
          : entry.facts?.length ? <dl className="mt-2.5 space-y-1 text-xs leading-5">{entry.facts.slice(0, 2).map((f, i) => <div key={i} className="flex gap-2"><dt className="w-16 shrink-0 text-faint">{f.label}</dt><dd className="line-clamp-1 min-w-0 flex-1 text-muted [overflow-wrap:anywhere]">{f.value}</dd></div>)}</dl> : null}
        <div className="mt-auto pt-3">{actions(entry, true)}</div>
      </article> : <article id={entry.anchor} key={entry.id} className={clsx('collection-card group/card flex h-[28rem] min-w-0 scroll-mt-24 flex-col overflow-hidden rounded-xl border border-(--collection-border) bg-surface p-5', 'accent-' + (entry.category || 'none'), entry.className, highlight(entry) && ring)}>
        {entry.image && <div className="relative -mx-5 -mt-5 mb-4 h-36 shrink-0 overflow-hidden bg-night"><img src={entry.image.url} alt={entry.image.caption} loading="lazy" className="size-full object-cover transition-transform duration-500 group-hover/card:scale-[1.04] motion-reduce:transition-none" /><span className="absolute inset-0 bg-gradient-to-t from-black/40 to-transparent" /></div>}
        <div className="flex h-8 shrink-0 items-center gap-3">{!entry.image && <IconTile category={entry.category || ''} size="sm" />}<span className={clsx('truncate rounded-full border border-line bg-paper/60 px-2.5 py-0.5 text-[11px] leading-5 text-chip-ink', !entry.image && 'ml-auto')} title={entry.kind}>{entry.kind}</span></div>
        <h2 className="card-title mt-3 h-14 shrink-0 font-serif text-[19px] leading-7 font-bold tracking-[-.01em] transition-colors [overflow-wrap:anywhere]" title={entry.title}><span className="line-clamp-2">{entry.title}</span></h2>
        <p className={clsx('mt-2 shrink-0 text-[13.5px] leading-6 text-ink/85 [overflow-wrap:anywhere]', entry.image ? 'line-clamp-2 h-12' : entry.tags?.length ? 'line-clamp-5 h-30' : 'line-clamp-6 h-36')}>{entry.summary}</p>
        {entry.tags?.length ? <div className="mt-4 h-[3.75rem] shrink-0 overflow-hidden"><TagList tags={entry.tags} category={entry.category} compact /></div>
          : entry.facts?.length ? <dl className="mt-4 shrink-0 space-y-1.5 text-[12px] leading-5">{entry.facts.slice(0, entry.image ? 2 : 3).map((f, i) => <div key={i} className="flex gap-2"><dt className="w-16 shrink-0 text-faint">{f.label}</dt><dd className="line-clamp-1 min-w-0 flex-1 text-muted [overflow-wrap:anywhere]">{f.value}</dd></div>)}</dl>
          : entry.status ? <p className="mt-4 line-clamp-2 shrink-0 text-xs leading-5 text-faint">합의·제안·미정 항목은 빠르게 보기에서 확인하세요.</p> : null}
        <div className="mt-auto">{actions(entry, true)}</div>
      </article>)}</div> : view === 'table' ?
      <div className="panel max-w-full overflow-x-auto" role="region" aria-label={label + ' 비교표'} tabIndex={0}><table className="w-full min-w-[760px] border-collapse text-left text-[13px]"><caption className="border-b border-line px-5 py-3 text-left text-xs text-muted">{label} 비교 · 좁은 화면에서는 표를 좌우로 이동할 수 있습니다.</caption><thead className="bg-tint/60">
        <tr>{['이름', '구분', '내용·효과', '주요 조건·정보'].map(t => <th scope="col" key={t} className="px-5 py-3 font-mono text-[10.5px] font-medium tracking-[.12em] text-faint">{t}</th>)}</tr></thead>
        <tbody>{entries.map(entry => <tr id={entry.anchor} key={entry.id} className={clsx('collection-table-row scroll-mt-24 border-t border-line align-top transition-colors hover:bg-tint/40', entry.className, highlight(entry) && ring)}>
          <th scope="row" className="w-[24%] px-5 py-5 font-normal"><span className="flex items-start gap-3"><IconTile category={entry.category || ''} size="sm" /><span className="min-w-0"><span className="block pt-1 text-[14px] leading-6 font-semibold">{title(entry)}</span>{actions(entry)}</span></span></th>
          <td className="w-[12%] px-5 py-5"><span className="inline-block rounded-full border border-line px-2.5 py-0.5 text-[11px] text-chip-ink">{entry.kind}</span></td>
          <td className="w-[38%] px-5 py-5 leading-6">{entry.summary}<div className="mt-3"><TagList tags={entry.tags} category={entry.category} /></div><Status entry={entry} /></td>
          <td className="px-5 py-5">{entry.facts?.length ? <Facts entry={entry} /> : <span className="text-faint">상세 내용 참조</span>}</td></tr>)}</tbody></table></div> :
      <div className="panel overflow-hidden">{entries.map(entry => <details id={entry.anchor} open={highlight(entry) || undefined} key={entry.id} className={clsx('collection-list-row group/row scroll-mt-24 border-b border-line last:border-b-0', entry.className, highlight(entry) && ring)}>
        <summary className="flex items-center gap-3.5 px-5 py-3.5 transition-colors hover:bg-tint/50 [&::-webkit-details-marker]:hidden"><IconTile category={entry.category || ''} size="sm" /><span className="min-w-0 flex-1 text-[14px] font-semibold">{entry.title}</span><span className="hidden max-w-40 truncate text-[11px] text-muted sm:block">{entry.kind}</span><ChevronRight className="size-4 shrink-0 text-faint transition-transform group-open/row:rotate-90" aria-hidden="true" /></summary>
        <div className="border-t border-line bg-paper/40 px-5 py-5 sm:pl-[4.25rem]"><p className="mb-4 text-[14px] leading-7">{entry.summary}</p><div className="mb-4"><TagList tags={entry.tags} category={entry.category} /></div><Facts entry={entry} /><Status entry={entry} />{actions(entry)}</div></details>)}</div>}
    <Dialog.Root open={selected !== null} onOpenChange={open => { if (!open) setSelected(null); }}><Dialog.Portal><Dialog.Overlay className="fixed inset-0 z-40 bg-black/55 backdrop-blur-[3px] data-[state=open]:animate-overlay-in" /><Dialog.Content className="fixed top-1/2 left-1/2 z-50 flex max-h-[90dvh] w-[calc(100%_-_2rem)] max-w-4xl -translate-x-1/2 -translate-y-1/2 flex-col overflow-hidden rounded-2xl border border-line bg-paper shadow-[0_30px_80px_-20px_rgb(0_0_0/.55)] data-[state=open]:animate-dialog-in" aria-describedby={undefined} onCloseAutoFocus={e => { e.preventDefault(); opener.current?.focus(); }}>
      <div className="flex shrink-0 items-center gap-4 border-b border-line bg-surface px-5 py-4 md:px-7">{selected && <IconTile category={selected.category || ''} />}<div className="min-w-0 flex-1"><p className="kicker mb-0.5">{label} · 빠르게 보기</p><Dialog.Title className="truncate font-serif text-xl font-bold">{selected?.title}</Dialog.Title></div><Dialog.Close className="icon-button" aria-label="빠르게 보기 닫기"><X className="size-4" /></Dialog.Close></div>
      <div className="min-h-0 overflow-y-auto overscroll-contain p-5 md:p-8">{selected && <TagDetails tags={selected.tags} category={selected.category} />}{selected && (selected.pageId ? <DocumentPreview key={selected.id} id={selected.pageId} /> : <>{selected.image && <img className="mb-6 max-h-80 w-full rounded-lg border border-line bg-night object-contain" src={selected.image.url} alt={selected.image.caption} />}<p className="mb-6 text-[15px] leading-7">{selected.summary}</p><Facts entry={selected} /><Status entry={selected} />{selected.detail}</>)}</div>
      {selected?.href && <div className="flex shrink-0 justify-end border-t border-line bg-surface px-5 py-3 md:px-7"><Link to={selected.href} className="button-primary">전체 내용으로 이동<ArrowRight className="size-3.5" /></Link></div>}
    </Dialog.Content></Dialog.Portal></Dialog.Root>
  </>;
}
