import { ArrowUpRight, BookOpen, ChevronRight, Diamond, Gem, History, LayoutGrid, Map, Network, Sparkles, Swords } from 'lucide-react';
import type { LucideIcon } from 'lucide-react';
import { Link } from 'react-router';
import clsx from 'clsx';
import { useData } from '../data';
import type { PageMeta, Scene } from '../types';
import type { ReactNode } from 'react';

export const icons: Record<string, LucideIcon> = { abnormalities: Diamond, fragments: Sparkles, equipment: Swords, relics: Gem, tags: Network, rules: BookOpen, principles: LayoutGrid, ui: Map, history: History };
export function Icon({ category, className = 'size-5' }: { category: string; className?: string }) {
  const Component = icons[category] || BookOpen;
  return <Component className={className} strokeWidth={1.5} aria-hidden="true" />;
}
/** Category icon on a tinted square; the accent hue comes from `.accent-<category>`. */
export function IconTile({ category, size = 'md' }: { category: string; size?: 'sm' | 'md' | 'lg' }) {
  return <span className={clsx('icon-tile', 'accent-' + category, { sm: 'size-8', md: 'size-10', lg: 'size-14 rounded-xl' }[size])}>
    <Icon category={category} className={{ sm: 'size-4', md: 'size-5', lg: 'size-7' }[size]} />
  </span>;
}
export function Breadcrumb({ items }: { items: { title: string; to?: string }[] }) {
  return <nav aria-label="현재 위치" className="mb-7 flex flex-wrap items-center gap-1.5 text-xs text-muted"><Link to="/" className="transition-colors hover:text-brand">위키 홈</Link>{items.map((i, n) => <span key={n} className="inline-flex items-center gap-1.5"><ChevronRight className="size-3 text-faint" aria-hidden="true" />{i.to ? <Link to={i.to} className="transition-colors hover:text-brand">{i.title}</Link> : <span className="text-ink">{i.title}</span>}</span>)}</nav>;
}
export function SectionHeading({ title, eyebrow, children }: { title: string; eyebrow?: string; children?: ReactNode }) {
  return <div className="mb-5 mt-16 flex flex-wrap items-end justify-between gap-3">
    <div>{eyebrow && <p className="mono-label mb-2">{eyebrow}</p>}<h2 className="font-serif text-[24px] leading-tight font-bold tracking-[-.01em]">{title}</h2></div>
    <div className="text-xs text-muted">{children}</div>
  </div>;
}
const badgeTone: Record<string, string> = {
  confirmed: 'bg-[#e7f1e8] text-[#2d6b3c] dark:bg-[#15281b] dark:text-[#93d3a2]',
  default: 'bg-[#e8eff7] text-[#2b5683] dark:bg-[#16222f] dark:text-[#9ec3ea]',
  draft: 'bg-[#f1ebf7] text-[#66398c] dark:bg-[#251a30] dark:text-[#cfaff0]',
  pending: 'bg-[#f8edd6] text-[#865a0e] dark:bg-[#2d2412] dark:text-[#ebc675]',
  deferred: 'bg-[#efe8dc] text-[#6a6355] dark:bg-[#242119] dark:text-[#bdb4a1]',
};
export function Badge({ status }: { status: string }) {
  const { meta } = useData();
  return <span className={clsx('inline-flex items-center gap-1.5 rounded-full px-2 py-0.5 text-[11px] leading-5 font-medium', badgeTone[status] || badgeTone.deferred)}>
    <span className="size-1.5 rounded-full bg-current" aria-hidden="true" />{meta.statuses[status] || status}
  </span>;
}
export function Empty({ children }: { children: ReactNode }) {
  return <div className="rounded-xl border border-dashed border-line-strong bg-surface/50 px-6 py-14 text-center text-sm leading-7 text-muted">{children}</div>;
}
export function PageRow({ page, excerpt }: { page: PageMeta; excerpt?: string }) {
  const { meta } = useData();
  const category = meta.categories.find(c => c.id === page.category);
  return <Link to={page.path} className="page-row group flex items-center gap-4 border-b border-line bg-surface px-4 py-4 transition-colors hover:bg-tint md:px-5">
    <IconTile category={page.category} />
    <span className="min-w-0 flex-1"><strong className="text-sm font-semibold md:text-base">{page.title}</strong><span className="mt-1 block text-xs leading-relaxed text-muted">{excerpt || [page.tier, category?.title].filter(Boolean).join(' · ')}</span></span>
    <ArrowUpRight className="size-4 shrink-0 text-muted transition-transform group-hover:-translate-y-0.5 group-hover:translate-x-0.5" aria-hidden="true" />
  </Link>;
}
export function SceneCard({ scene, actions = scene.actions }: { scene: Scene; actions?: Scene['actions'] }) {
  const { meta } = useData();
  return <article className="scene-card group flex h-[28rem] min-w-0 flex-col overflow-hidden rounded-xl border border-line bg-surface shadow-(--shadow-card) transition-[border-color,box-shadow] hover:border-line-strong hover:shadow-(--shadow-lift)">
    <Link to={'/scenes/' + scene.id + '/'} tabIndex={-1} aria-hidden="true" className="relative block overflow-hidden bg-night">
      <img src={scene.images[0].url} alt="" loading="lazy" className="aspect-[2.1] w-full object-cover transition-transform duration-500 group-hover:scale-[1.03] motion-reduce:transition-none" />
      <span className="absolute inset-0 bg-gradient-to-t from-black/55 via-transparent to-transparent" />
      <span className="absolute bottom-3 left-4 rounded-full bg-black/40 px-2.5 py-0.5 text-[11px] text-white/90 backdrop-blur-sm">{meta.groups.find(g => g.id === scene.group)?.title} · {scene.kind}</span>
    </Link>
    <div className="flex min-h-0 flex-1 flex-col p-5">
      <h3 className="line-clamp-2 shrink-0 font-serif text-[19px] leading-7 font-bold"><Link to={'/scenes/' + scene.id + '/'} className="transition-colors hover:text-brand">{scene.title}</Link></h3>
      <p className="mt-1.5 line-clamp-2 h-10 shrink-0 text-[13px] leading-5 text-muted">{scene.subtitle}</p>
      <p className="kicker mt-auto pt-4">행동 {actions.length === scene.actions.length ? scene.actions.length : `${actions.length} / ${scene.actions.length}`}개</p>
      <div className="mt-2 flex flex-wrap gap-1.5">{actions.map(a => <Link className="chip action-link" key={a.id} to={`/scenes/${scene.id}/#action-${a.id}`}>{a.title}</Link>)}</div>
    </div>
  </article>;
}
