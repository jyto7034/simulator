import { useEffect, useId, useMemo, useRef, useState } from 'react';
import { Link, NavLink, Outlet, useLocation, useNavigate } from 'react-router';
import { Dialog } from 'radix-ui';
import { ArrowUpRight, CornerDownLeft, Home, Menu, Search, X, Printer, Map, PanelsTopLeft } from 'lucide-react';
import { categoryPages, useData } from '../data';
import { Icon, IconTile } from './common';
import { ThemeToggle } from './ThemeToggle';
import clsx from 'clsx';

export function Mark({ className = 'size-7' }: { className?: string }) {
  return <svg viewBox="0 0 32 32" className={className} fill="none" aria-hidden="true">
    <path d="M16 2.5 29.5 16 16 29.5 2.5 16Z" stroke="currentColor" strokeWidth="1.4" />
    <path d="M16 8.5 23.5 16 16 23.5 8.5 16Z" stroke="currentColor" strokeWidth="1.1" opacity=".55" />
    <circle cx="16" cy="16" r="2.2" fill="currentColor" />
  </svg>;
}

function Navigation({ onNavigate }: { onNavigate?: () => void }) {
  const { meta } = useData();
  const item = (id: string) => {
    const c = meta.categories.find(c => c.id === id)!;
    return <NavLink key={id} to={id==='history'?'/wiki/archive/':'/categories/' + id + '/'} onClick={onNavigate} className={({ isActive }) => clsx('nav-link', isActive && 'nav-active')}><Icon category={id} /><span>{c.title}</span><span className="nav-count">{categoryPages(meta.pages, id).length}</span></NavLink>;
  };
  return <div className="flex h-full flex-col px-5 pt-7 pb-6">
    <Link to="/" onClick={onNavigate} className="group flex items-center gap-3 px-3">
      <Mark className="size-8 text-[#d6a85a] transition-transform duration-500 group-hover:rotate-45 motion-reduce:transition-none" />
      <span><span className="block font-serif text-[26px] leading-none font-bold tracking-[.14em] text-[#f3ead6]">ARK</span><span className="mt-1.5 block font-mono text-[9px] tracking-[.24em] text-[#8f8572]">GAME ENCYCLOPEDIA</span></span>
    </Link>
    <div className="mx-3 mt-7 mb-3 h-px bg-gradient-to-r from-[#d6a85a]/40 via-white/10 to-transparent" />
    <nav aria-label="위키 탐색"><NavLink end to="/" onClick={onNavigate} className={({ isActive }) => clsx('nav-link', isActive && 'nav-active')}><Home strokeWidth={1.5} />위키 홈</NavLink>
      <p className="nav-label">콘텐츠 도감</p>{['abnormalities', 'fragments', 'equipment', 'relics'].map(item)}
      <p className="nav-label">플레이 가이드</p><NavLink to="/scenes/" onClick={onNavigate} className={({ isActive }) => clsx('nav-link', isActive && 'nav-active')}><Map strokeWidth={1.5} />장면·플레이어 행동<span className="nav-count">{meta.scenes.length}</span></NavLink>{['rules', 'ui'].map(item)}
      <NavLink to="/screens/" onClick={onNavigate} className={({ isActive }) => clsx('nav-link', isActive && 'nav-active')}><PanelsTopLeft strokeWidth={1.5} />전체 화면 목록<span className="nav-count">{meta.catalog.length}</span></NavLink>
      <p className="nav-label">설계와 기록</p>{['principles', 'history'].map(item)}
    </nav>
    <div className="mt-auto px-3 pt-10">
      <div className="rounded-lg border border-white/[.07] bg-white/[.025] p-3.5 text-[11px] leading-5 text-[#9a917e]">
        <p className="mb-1 font-mono text-[9.5px] tracking-[.18em] text-[#d6a85a]/80">INTERNAL DESIGN WIKI</p>
        최신 합의를 표시합니다. 이전 합의·검토안은 아카이브에 보존합니다. 설계와 구현 여부는 별개입니다.
      </div>
    </div>
  </div>;
}

interface QuickResult { key: string; title: string; hint: string; to: string; category: string }

/** Header search: jumps straight to matching titles while typing; Enter without a selection opens full-text search. */
function QuickSearch() {
  const { meta } = useData();
  const location = useLocation(), navigate = useNavigate();
  const [query, setQuery] = useState('');
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(-1);
  const input = useRef<HTMLInputElement>(null);
  const listId = useId();
  useEffect(() => {
    if (location.pathname.startsWith('/search')) setQuery(new URLSearchParams(location.search).get('q') || '');
    setOpen(false);
  }, [location.pathname, location.search]);
  useEffect(() => {
    function key(e: KeyboardEvent) {
      if (e.key === '/' && !(e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement || e.target instanceof HTMLSelectElement)) {
        e.preventDefault(); input.current?.focus();
      }
    }
    document.addEventListener('keydown', key); return () => document.removeEventListener('keydown', key);
  }, []);
  const results = useMemo<QuickResult[]>(() => {
    const terms = query.trim().toLowerCase().split(/\s+/).filter(Boolean);
    if (!terms.length) return [];
    const hit = (title: string) => terms.every(t => title.toLowerCase().includes(t));
    const scenes = meta.scenes.filter(s => hit(s.title)).map(s => ({ key: 'scene:' + s.id, title: s.title, hint: '장면 · ' + s.kind, to: '/scenes/' + s.id + '/', category: 'ui' }));
    const pages = meta.pages.filter(p => p.category !== 'history' && hit(p.title))
      .sort((a, b) => Number(b.title.toLowerCase().startsWith(terms[0])) - Number(a.title.toLowerCase().startsWith(terms[0])) || a.title.length - b.title.length)
      .map(p => ({ key: p.id, title: p.title, hint: [meta.categories.find(c => c.id === p.category)?.title, p.tier].filter(Boolean).join(' · '), to: p.path, category: p.category }));
    return [...scenes, ...pages].slice(0, 7);
  }, [query, meta]);
  const expanded = open && query.trim().length > 0;
  function submit() { setOpen(false); input.current?.blur(); navigate('/search/?q=' + encodeURIComponent(query)); }
  function go(r: QuickResult) { setOpen(false); input.current?.blur(); navigate(r.to); }
  return <form role="search" className="relative ml-auto w-full max-w-xl" onSubmit={e => { e.preventDefault(); if (expanded && active >= 0 && results[active]) go(results[active]); else submit(); }}>
    <Search className="pointer-events-none absolute top-1/2 left-3.5 size-4 -translate-y-1/2 text-faint" aria-hidden="true" />
    <input ref={input} type="search" role="combobox" aria-label="위키 전체 검색" aria-expanded={expanded} aria-controls={listId} aria-autocomplete="list"
      aria-activedescendant={expanded && active >= 0 ? listId + '-' + active : undefined}
      placeholder="환상체·스킬·유물·행동 검색…" value={query}
      onChange={e => { setQuery(e.target.value); setOpen(true); setActive(-1); }} onFocus={() => setOpen(true)} onBlur={() => setOpen(false)}
      onKeyDown={e => {
        if (e.key === 'Escape' && expanded) { e.preventDefault(); setOpen(false); return; }
        if (!expanded || !results.length) return;
        if (e.key === 'ArrowDown') { e.preventDefault(); setActive(a => (a + 1) % results.length); }
        if (e.key === 'ArrowUp') { e.preventDefault(); setActive(a => (a <= 0 ? results.length : a) - 1); }
      }}
      className="field peer w-full py-2.5 pr-24 pl-10 text-[13px] md:text-sm" />
    <span className="pointer-events-none absolute top-1/2 right-16 hidden -translate-y-1/2 md:peer-placeholder-shown:inline-grid md:peer-focus:hidden"><span className="kbd">/</span></span>
    <button type="submit" aria-label="검색 실행" className="absolute top-1/2 right-1.5 -translate-y-1/2 rounded-md px-2.5 py-1.5 text-xs font-medium text-brand transition-colors hover:bg-tint">검색</button>
    <div hidden={!expanded} className="absolute inset-x-0 top-full z-30 mt-2 overflow-hidden rounded-xl border border-line bg-raised shadow-(--shadow-lift)">
      <div id={listId} role="listbox" aria-label="바로 가기">
      {results.map((r, i) => <div key={r.key} id={listId + '-' + i} role="option" aria-selected={i === active} onMouseDown={e => e.preventDefault()} onMouseEnter={() => setActive(i)} onClick={() => go(r)}
        className={clsx('flex cursor-pointer items-center gap-3 px-3 py-2.5', i === active && 'bg-tint')}>
        <IconTile category={r.category} size="sm" /><span className="min-w-0 flex-1"><span className="block truncate text-[13px] font-medium">{r.title}</span><span className="block truncate text-[11px] text-muted">{r.hint}</span></span>
        {i === active && <CornerDownLeft className="size-3.5 text-faint" aria-hidden="true" />}
      </div>)}
      </div>
      <div className="flex items-center justify-between gap-3 border-t border-line bg-surface px-3.5 py-2.5 text-[11px] text-muted">
        <span>{results.length ? '↑↓ 이동 · Enter 열기' : '제목이 일치하는 문서가 없습니다'}</span>
        <button type="submit" onMouseDown={e => e.preventDefault()} className="font-medium text-brand hover:underline">‘{query.trim()}’ 본문까지 검색 →</button>
      </div>
    </div>
  </form>;
}

export function Layout() {
  const [open, setOpen] = useState(false);
  const location = useLocation();
  useEffect(() => {
    window.scrollTo({ top: 0, behavior: 'instant' });
  }, [location.pathname]);
  return <>
    <a className="skip-link" href="#main">본문으로 이동</a>
    <aside className="fixed inset-y-0 left-0 z-20 hidden w-64 overflow-y-auto border-r border-black/40 bg-night text-[#d4cbb8] lg:block">
      <div className="pointer-events-none absolute inset-x-0 top-0 h-56 bg-[radial-gradient(240px_160px_at_30%_0%,rgb(214_168_90/.10),transparent)]" />
      <Navigation />
    </aside>
    <div className="min-h-screen lg:ml-64">
      <header className="sticky top-0 z-20 flex min-h-16 items-center gap-3 border-b border-line bg-paper/85 px-4 backdrop-blur-md backdrop-saturate-150 md:px-8 xl:px-12">
        <Dialog.Root open={open} onOpenChange={setOpen}><Dialog.Trigger asChild><button className="icon-button lg:hidden" aria-label="탐색 메뉴 열기"><Menu className="size-5" /></button></Dialog.Trigger>
          <Dialog.Portal><Dialog.Overlay className="fixed inset-0 z-40 bg-black/60 backdrop-blur-sm data-[state=open]:animate-overlay-in" /><Dialog.Content className="fixed inset-y-0 left-0 z-50 w-72 max-w-[85vw] overflow-y-auto bg-night text-[#d4cbb8] shadow-2xl data-[state=open]:animate-drawer-in" aria-describedby={undefined}>
            <Dialog.Title className="sr-only">위키 탐색 메뉴</Dialog.Title><Dialog.Close className="absolute top-4 right-3 rounded-md p-2 text-white/60 hover:bg-white/5 hover:text-white" aria-label="탐색 메뉴 닫기"><X className="size-5" /></Dialog.Close><Navigation onNavigate={() => setOpen(false)} />
          </Dialog.Content></Dialog.Portal>
        </Dialog.Root>
        <Link to="/" className="flex items-center gap-2 lg:hidden" aria-label="위키 홈"><Mark className="size-6 text-brass" /><span className="hidden font-serif text-lg font-bold tracking-[.12em] sm:inline">ARK</span></Link>
        <QuickSearch />
        <ThemeToggle />
        <button className="icon-button hidden sm:grid" onClick={() => window.print()} aria-label="현재 문서 인쇄"><Printer className="size-4" /></button>
      </header>
      <main id="main" tabIndex={-1} className="mx-auto min-h-[80vh] max-w-[84rem] px-5 py-9 outline-none md:px-8 xl:px-12"><Outlet /></main>
      <footer className="mx-auto max-w-[84rem] px-5 md:px-8 xl:px-12">
        <div className="flex flex-wrap items-center justify-between gap-4 border-t border-line py-7 text-[12px] text-muted">
          <span className="flex items-center gap-2.5"><Mark className="size-4 text-brass" />ARK 게임 위키 · 콘텐츠와 플레이 경험을 연결하는 설계 도감</span>
          <Link className="inline-flex items-center gap-1 transition-colors hover:text-brand" to="/about/">출처와 갱신 방법<ArrowUpRight className="size-3" /></Link>
        </div>
      </footer>
    </div>
  </>;
}
