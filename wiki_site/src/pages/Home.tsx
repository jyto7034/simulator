import { ArrowRight, ArrowUpRight, Swords, Tent, Wrench, Map as MapIcon } from 'lucide-react';
import { Link } from 'react-router';
import { categoryPages, pathFor, useData } from '../data';
import { IconTile, SceneCard, SectionHeading } from '../components/common';

export function HomePage() {
  const { meta } = useData(), rest = meta.scenes.find(s => s.id === 'rest')!;
  const count = (id: string) => categoryPages(meta.pages, id).length;
  const stats: [number, string][] = [[count('abnormalities'), '환상체·도구'], [count('relics'), '유물 항목'], [meta.groups.length, '상세 장면'], [meta.scenes.reduce((n, s) => n + s.actions.length, 0), '플레이어 행동']];
  const flow = [['rest', '세이프존 준비', Tent], ['map', '지도·노드 이동', MapIcon], ['battle', '전투', Swords], ['maintenance', '메인터넌스', Wrench]] as const;
  return <>
    <section className="relative isolate -mt-2 overflow-hidden rounded-2xl bg-night text-[#f1e9d8] shadow-[0_30px_60px_-30px_rgb(0_0_0/.6)] ring-1 ring-black/20">
      <figure className="absolute inset-0 -z-10 m-0">
        <img src={rest.images[0].url} alt="직원들이 모인 세이프존 휴식 화면 시안" fetchPriority="high" className="absolute right-0 bottom-0 h-[115%] w-full object-cover object-[50%_100%] md:h-[130%] md:w-[66%] md:object-[60%_100%]" />
        <span className="absolute inset-0 bg-[linear-gradient(0deg,#0c0b09_0%,rgb(12_11_9/.95)_62%,rgb(12_11_9/.35)_100%)] md:bg-[linear-gradient(90deg,#0c0b09_0%,#0c0b09_34%,rgb(12_11_9/.82)_46%,rgb(12_11_9/.35)_70%,rgb(12_11_9/.05)_100%)]" />
        <span className="absolute inset-x-0 bottom-0 h-40 bg-gradient-to-t from-[#0c0b09]/90 to-transparent" />
        <figcaption className="absolute top-4 right-5 rounded-full bg-black/45 px-3 py-1 text-[11px] text-white/75 backdrop-blur-sm">세이프존 · 휴식 화면 시안</figcaption>
      </figure>
      <div className="px-6 pt-48 pb-8 md:max-w-[38rem] md:px-12 md:pt-16 md:pb-10">
        <p className="eyebrow !text-[#d6a85a]">A FIELD GUIDE TO ARK</p>
        <h1 className="mt-5 font-serif text-[36px] leading-[1.2] font-bold tracking-[-.02em] text-balance md:text-[50px]"><span className="mb-1 block text-[.5em] font-semibold tracking-[.32em] text-[#d6a85a]">ARK</span>세계와 플레이를 읽다.</h1>
        <p className="mt-5 max-w-md text-[15px] leading-7 text-[#cfc5b1]">환상체를 만나고, 능력을 조합하고, 다음 행동을 고르기까지. 콘텐츠의 상세 정보와 장면별 플레이 방법을 한곳에서 찾아보세요.</p>
        <div className="mt-8 flex flex-wrap items-center gap-3">
          <Link className="inline-flex items-center gap-2.5 rounded-lg bg-[#d6a85a] px-5 py-2.5 text-[13px] font-semibold text-[#17130b] shadow-[0_6px_20px_-6px_rgb(214_168_90/.6)] transition-colors hover:bg-[#e6bb6f]" to="/categories/abnormalities/">환상체 도감<ArrowRight className="size-4" /></Link>
          <Link className="inline-flex items-center gap-1.5 rounded-lg border border-white/20 px-4 py-2.5 text-[13px] text-[#efe6d3] backdrop-blur-sm transition-colors hover:border-[#d6a85a] hover:text-[#f6dca8]" to="/scenes/">플레이 가이드<ArrowUpRight className="size-4" /></Link>
        </div>
      </div>
      <dl className="grid grid-cols-2 border-t border-white/10 bg-black/30 backdrop-blur-sm sm:grid-cols-4">{stats.map(([n, t], i) => <div key={t} className={'flex items-baseline gap-2.5 border-white/10 px-6 py-4 md:px-12 ' + (i % 2 ? 'border-l ' : '') + (i === 2 ? 'max-sm:border-t sm:border-l ' : i === 3 ? 'max-sm:border-t ' : '')}>
        <dt className="order-2 text-[12px] text-[#a99f8a]">{t}</dt><dd className="order-1 font-serif text-[28px] leading-none font-bold text-[#f3e3bf] tabular-nums">{n}</dd>
      </div>)}</dl>
    </section>

    <SectionHeading eyebrow="ENCYCLOPEDIA" title="콘텐츠 도감">항목을 고르면 관련 능력과 규칙으로 이어집니다.</SectionHeading>
    <div className="directory grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">{meta.categories.slice(0, 5).map((c, i) => <Link key={c.id} to={c.id==='history'?'/wiki/archive/':'/categories/' + c.id + '/'} className={'group relative flex min-h-40 flex-col overflow-hidden rounded-xl border border-line bg-surface p-5 shadow-(--shadow-card) transition-[transform,border-color,box-shadow] duration-200 hover:-translate-y-0.5 hover:border-(--accent) hover:shadow-(--shadow-lift) motion-reduce:transform-none md:p-6 accent-' + c.id}>
      <span className="absolute inset-x-0 top-0 h-[2px] bg-gradient-to-r from-(--accent) to-transparent opacity-0 transition-opacity group-hover:opacity-100" />
      <div className="flex items-start justify-between gap-3"><IconTile category={c.id} /><span className="font-mono text-[11px] text-faint tabular-nums">{String(i + 1).padStart(2, '0')}</span></div>
      <strong className="mt-5 font-serif text-[19px] font-bold">{c.title}</strong>
      <p className="mt-1.5 line-clamp-2 text-[13px] leading-relaxed text-muted">{c.description}</p>
      <span className="mt-auto flex items-center justify-between pt-4 text-xs text-muted"><span><b className="font-semibold text-ink tabular-nums">{count(c.id)}</b> 항목</span><ArrowUpRight className="size-4 text-faint transition-[transform,color] group-hover:translate-x-0.5 group-hover:-translate-y-0.5 group-hover:text-(--accent)" aria-hidden="true" /></span>
    </Link>)}</div>
    <div className="mt-3 flex flex-wrap gap-2 text-[13px]">{meta.categories.slice(5).map(c => <Link key={c.id} to={c.id==='history'?'/wiki/archive/':'/categories/' + c.id + '/'} className="inline-flex items-center gap-2.5 rounded-lg border border-line bg-surface py-1.5 pr-3.5 pl-1.5 transition-colors hover:border-brand hover:text-brand"><IconTile category={c.id} size="sm" />{c.title}<span className="font-mono text-[11px] text-faint">{count(c.id)}</span></Link>)}</div>

    <SectionHeading eyebrow="PLAYER'S FIELD GUIDE" title="어디에서, 무엇을 할 수 있나"><Link to="/scenes/" className="inline-flex items-center gap-1 transition-colors hover:text-brand">{meta.scenes.length}개 화면·모드 모두 보기<ArrowRight className="size-3.5" /></Link></SectionHeading>
    <ol className="panel mb-5 flex flex-wrap items-center gap-y-3 px-4 py-3.5 text-[13px] md:px-5">{flow.map(([id, t, FlowIcon], i) => <li key={id} className="flex items-center">
      {i > 0 && <span className="mx-2 font-mono text-xs text-faint md:mx-3" aria-hidden="true">{i === 1 ? '⇄' : i === 3 ? '/' : '→'}</span>}
      <Link className="group inline-flex items-center gap-2.5 rounded-lg px-2 py-1.5 transition-colors hover:bg-tint" to={'/scenes/' + id + '/'}><span className="grid size-7 place-items-center rounded-md bg-tint text-brand transition-colors group-hover:bg-brand group-hover:text-paper"><FlowIcon className="size-3.5" strokeWidth={1.75} /></span><span className="font-medium">{t}</span></Link>
    </li>)}<li className="w-full border-t border-line pt-3 text-[12px] text-muted md:ml-auto md:w-auto md:border-0 md:pt-0">활동을 마치면 세이프존으로 돌아옵니다. 휴식과 지도는 같은 장면의 보기입니다.</li></ol>
    <div className="grid gap-4 md:grid-cols-3">{['rest', 'maintenance', 'battle'].map(id => { const s = meta.scenes.find(s => s.id === id)!; return <SceneCard key={id} scene={s} actions={s.actions.slice(0, 3)} />; })}</div>

    <SectionHeading eyebrow="RECENT DOSSIERS" title="이어서 살펴볼 콘텐츠">최근 합의가 반영된 설계</SectionHeading>
    <div className="grid gap-4 md:grid-cols-3">{[['knight-of-despair', '절망의 기사', '검과 가호, 절망 상태의 연속기.'], ['queen-of-hatred', '증오의 여왕', '하트와 히스테릭, 회복과 공격의 연결.'], ['yin-yang', '음양', '두 영역의 순환 회복과 교집합 버프.']].map(([id, t, d], i) => <article key={id} className="accent-abnormalities group relative overflow-hidden rounded-xl border border-line bg-surface p-6 shadow-(--shadow-card)">
      <span className="pointer-events-none absolute -top-5 -right-1 font-serif text-[96px] leading-none font-bold text-(--accent) opacity-[.07] select-none" aria-hidden="true">{String(i + 1).padStart(2, '0')}</span>
      <p className="mono-label">ABNORMALITY / SKILL FRAGMENT</p>
      <h3 className="mt-3 font-serif text-[21px] font-bold"><Link to={pathFor('content/abnormalities/' + id + '/README')} className="transition-colors hover:text-brand">{t}</Link></h3>
      <p className="mt-2 text-[13.5px] leading-6 text-muted">{d}</p>
      <div className="mt-5 flex gap-2 text-xs"><Link className="chip" to={pathFor('content/abnormalities/' + id + '/fragment')}>파편 상세</Link><Link className="chip" to={pathFor('content/abnormalities/' + id + '/synergies')}>태그·유물</Link></div>
    </article>)}</div>
    <p className="mt-14 text-[12px] leading-relaxed text-muted">최신 합의와 남은 미정 사항을 표시하는 내부 설계 위키입니다. 구현 완료를 뜻하지 않습니다. 이전 합의와 검토안은 <Link className="text-brand underline decoration-brand/40 underline-offset-4 hover:decoration-brand" to="/wiki/archive/">아카이브</Link>에서 찾아볼 수 있습니다.</p>
  </>;
}
