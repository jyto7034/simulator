import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { render } from '../.ssr/entry-server.js';

const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const meta=JSON.parse(await fs.readFile(path.join(root,'public/generated/meta.json'),'utf8'));
const pages=JSON.parse(await fs.readFile(path.join(root,'.generated/pages.json'),'utf8'));
const template=await fs.readFile(path.join(root,'dist/index.html'),'utf8');
const base=meta.config.basePath.replace(/\/$/,'');
const routes=['/','/scenes/','/screens/','/search/','/about/','/wiki/archive/','/wiki/archive/search/',...meta.categories.map(c=>'/categories/'+c.id+'/'),...meta.scenes.map(s=>'/scenes/'+s.id+'/'),...meta.pages.map(p=>p.path)];
const escape=s=>String(s).replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
let totalBytes=0;
for(const route of [...routes,'/404.html']){
  const p=meta.pages.find(p=>p.path===route),s=meta.scenes.find(s=>'/scenes/'+s.id+'/'===route),c=meta.categories.find(c=>'/categories/'+c.id+'/'===route);
  const name=p?.title||s?.title||c?.title||({'/wiki/archive/':'아카이브','/wiki/archive/search/':'아카이브 검색','/scenes/':'장면·플레이어 행동','/screens/':'전체 화면 제작 목록','/search/':'통합 검색','/about/':'출처와 갱신','/404.html':'문서를 찾을 수 없습니다'})[route];
  const title=name?name+' | '+meta.title:meta.title,description=p?.description||s?.subtitle||c?.description||meta.config.description;
  const boot={meta,page:p?pages[p.id]:undefined,prerendered:true};
  const canonical=meta.config.canonicalOrigin?new URL(base+route,meta.config.canonicalOrigin).href:'';
  const seo=`<meta name="description" content="${escape(description)}"><meta name="robots" content="${meta.config.indexable&&route!=='/404.html'?'index,follow':'noindex,nofollow'}"><meta property="og:type" content="website"><meta property="og:title" content="${escape(title)}"><meta property="og:description" content="${escape(description)}">${canonical?`<link rel="canonical" href="${escape(canonical)}"><meta property="og:url" content="${escape(canonical)}">`:''}`;
  const data=JSON.stringify(boot).replace(/</g,'\\u003c');
  const result=template.replace('<title>ARK 게임 위키</title>','<title>'+escape(title)+'</title>').replace('<!--seo-->',seo).replace('<!--app-->',render(route,boot)).replace('<!--page-data-->',`<script type="application/json" id="wiki-boot">${data}</script>`);
  const target=path.join(root,'dist',route==='/404.html'?'404.html':route.slice(1),'index.html');
  const output=route==='/404.html'?path.join(root,'dist/404.html'):target;
  await fs.mkdir(path.dirname(output),{recursive:true});await fs.writeFile(output,result);totalBytes+=Buffer.byteLength(result);
}
// Keep bookmarks and inbound links, including their section fragments.
const redirects=[...meta.pages.filter(p=>p.legacyPath),{legacyPath:'/categories/tags/',path:'/wiki/content/tags/README/'}];
for(const p of redirects){
  const target=path.join(root,'dist',p.legacyPath.slice(1),'index.html');
  const destination=base+p.path;
  await fs.mkdir(path.dirname(target),{recursive:true});
  await fs.writeFile(target,`<!doctype html><html lang="ko"><meta charset="utf-8"><title>문서로 이동</title><meta http-equiv="refresh" content="0;url=${escape(destination)}"><script>location.replace(${JSON.stringify(destination)}+location.search+location.hash)</script><a href="${escape(destination)}">문서 보기</a></html>`);
}
await fs.writeFile(path.join(root,'dist/robots.txt'),meta.config.indexable?'User-agent: *\nAllow: /\n'+(meta.config.canonicalOrigin?'Sitemap: '+new URL(base+'/sitemap.xml',meta.config.canonicalOrigin).href+'\n':''):'User-agent: *\nDisallow: /\n');
if(meta.config.canonicalOrigin)await fs.writeFile(path.join(root,'dist/sitemap.xml'),'<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">'+routes.filter(r=>r!=='/search/').map(r=>'<url><loc>'+escape(new URL(base+r,meta.config.canonicalOrigin).href)+'</loc></url>').join('')+'</urlset>');
const report={status:'PASS',routes:routes.length,documents:meta.pages.length,ssg:true,basePath:meta.config.basePath,indexable:meta.config.indexable,htmlBytes:totalBytes};
await fs.writeFile(path.join(root,'.generated/build-report.json'),JSON.stringify(report,null,2)+'\n');
console.log(`Prerendered ${routes.length} routes + 404. Every document has an independent HTML page.`);
