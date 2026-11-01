import { chromium, expect } from '@playwright/test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import { createReadStream } from 'node:fs';
import http from 'node:http';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..'),dist=path.join(root,'dist');
const meta=JSON.parse(await fs.readFile(path.join(root,'public/generated/meta.json'),'utf8'));
const docs=JSON.parse(await fs.readFile(path.join(root,'.generated/pages.json'),'utf8'));
const base=meta.config.basePath.replace(/\/$/,'');
const artifacts=path.join(root,'artifacts');await fs.mkdir(artifacts,{recursive:true});
const mime={'.html':'text/html; charset=utf-8','.js':'text/javascript','.css':'text/css','.json':'application/json','.png':'image/png','.jpg':'image/jpeg','.webp':'image/webp','.svg':'image/svg+xml','.md':'text/plain; charset=utf-8','.csv':'text/csv; charset=utf-8'};
const server=http.createServer(async(req,res)=>{
 try{
  const url=new URL(req.url,'http://localhost'),requested=decodeURIComponent(url.pathname);
  if(!requested.startsWith(base+'/'))throw new Error('outside base');
  const rel=requested.slice(base.length);let file=path.resolve(dist,'.'+rel);
  if(!file.startsWith(dist+path.sep)&&file!==dist)throw new Error('outside dist');
  if((await fs.stat(file)).isDirectory())file=path.join(file,'index.html');
  const stat=await fs.stat(file);res.writeHead(200,{'Content-Type':mime[path.extname(file)]||'application/octet-stream','Content-Length':stat.size});createReadStream(file).pipe(res);
 }catch{res.writeHead(404,{'Content-Type':'text/html'});res.end(await fs.readFile(path.join(dist,'404.html')));}
});
await new Promise((resolve,reject)=>{server.once('error',reject);server.listen(0,'127.0.0.1',resolve);});
const origin='http://127.0.0.1:'+server.address().port,home=origin+base;
let browser;
try{
 browser=await chromium.launch({headless:true,args:['--no-sandbox']});
 const page=await browser.newPage({viewport:{width:1440,height:1100},deviceScaleFactor:1,colorScheme:'light'});
 const errors=[],requests=[],failed=[],consoleErrors=[];
 page.on('pageerror',e=>errors.push(e.message));page.on('requestfailed',r=>failed.push(r.url()));
 page.on('console',m=>{if(m.type()==='error')consoleErrors.push(m.text());});
 page.on('request',r=>{if(!r.url().startsWith(origin))requests.push(r.url());});
 const go=async route=>{const r=await page.goto(home+route);assert.equal(r.status(),200,route);await page.locator('main h1').waitFor();};
 const verifyCards=async()=>{
  const sizes=await page.locator('.collection-card').evaluateAll(cards=>cards.map(card=>{const box=card.getBoundingClientRect(),footer=card.querySelector('.collection-actions').getBoundingClientRect();return{profile:card.classList.contains('collection-profile'),width:box.width,height:box.height,bottom:box.bottom-footer.bottom,overflow:card.scrollHeight>card.clientHeight+1};}));
  assert(sizes.length>0);assert(Math.max(...sizes.map(s=>s.width))-Math.min(...sizes.map(s=>s.width))<=1);
  assert(sizes.every(s=>!s.overflow&&(s.profile ? s.height<330&&s.bottom>=16&&s.bottom<=18 : s.height===448&&s.bottom>=19&&s.bottom<=22)),JSON.stringify(sizes));
 };
 await go('/');await page.locator('figure img').evaluate(im=>im.decode());await page.screenshot({path:path.join(artifacts,'home-desktop.png'),fullPage:true,animations:'disabled'});
 assert.equal(await page.locator('.directory>a').count(),5);
 // The home payload must not download the full-text index or all document bodies.
 const loaded=await page.evaluate(()=>performance.getEntriesByType('resource').map(r=>r.name));assert(!loaded.some(u=>u.endsWith('/search.json')));
 await go('/categories/abnormalities/');assert.equal(await page.locator('.collection-card').count(),37);
 await page.getByLabel('분류 필터').selectOption('최종 보스');assert.equal(await page.locator('.collection-card').count(),4);
 await page.getByLabel('분류 필터').selectOption('');await page.getByLabel('목록 검색').fill('음양');assert.equal(await page.locator('.collection-card').count(),1);
 await page.locator('.collection-card').getByRole('link',{name:'상세 보기',exact:true}).click();await page.locator('#article-body').waitFor();
 await page.getByRole('navigation',{name:'관련 콘텐츠'}).getByRole('link',{name:'스킬 파편',exact:true}).click();await page.locator('#article-body').waitFor();
 assert((await page.locator('#article-body').textContent()).includes('순환 회복'));
 await page.reload();await page.locator('#article-body').waitFor();assert((await page.title()).includes('음양'));
 await page.screenshot({path:path.join(artifacts,'article-desktop.png'),fullPage:true,animations:'disabled'});
 await page.getByLabel('위키 전체 검색').fill('교대');await page.getByRole('button',{name:'검색 실행'}).click();await page.locator('.action-result').first().waitFor();
 await page.locator('.action-result').first().locator('a').first().click();await page.locator('.highlight').waitFor();assert(await page.locator('.highlight').isVisible());
 await page.locator('.highlight a').first().click();await page.locator('#article-body').waitFor();
 await go('/search/?q='+encodeURIComponent('깨진 맹세'));await page.waitForFunction(()=>document.querySelector('[role="status"]')?.textContent.includes('검색 결과'));
 assert.equal(await page.locator('.document-result').count(),0);
 await page.getByRole('link',{name:'아카이브에서 검색',exact:true}).click();await expect(page.locator('main h1')).toHaveText('아카이브 검색');await page.locator('.document-result').first().waitFor();
 assert(page.url().includes('/wiki/archive/search/'));
 const currentIndex=JSON.parse(await fs.readFile(path.join(root,'public/generated/search.json'),'utf8'));
 const archiveIndex=JSON.parse(await fs.readFile(path.join(root,'public/generated/archive-search.json'),'utf8'));
 assert(currentIndex.every(p=>p.category!=='history'));assert(archiveIndex.every(p=>p.category==='history'));
 assert(!currentIndex.some(p=>p.id==='content/relics/weapon-stat-bonuses'));
 assert(archiveIndex.some(p=>p.id==='content/relics/weapon-stat-bonuses'));
 assert(!docs['content/relics/borrowed-life'].html.includes('받는 피해 <code>+50%'));
 assert(Object.values(docs).some(p=>p.archived&&p.html.includes('받는 피해 <code>+50%')));
 await go('/wiki/content/abnormalities/knight-of-despair/fragment/');
 assert.equal(await page.getByRole('navigation',{name:'관련 콘텐츠'}).getByRole('link',{name:'검토 기록',exact:true}).count(),0);
 await go('/wiki/content/abnormalities/knight-of-despair/review/');
 assert(page.url().includes('/wiki/archive/'));await expect(page.locator('.notice')).toContainText('현재 규칙으로 사용하지 않습니다');
 await go('/wiki/archive/');await expect(page.locator('main h1')).toHaveText('아카이브');
 await page.screenshot({path:path.join(artifacts,'archive-desktop.png'),fullPage:true,animations:'disabled'});
 await go('/wiki/archive/search/?q='+encodeURIComponent('깨진 맹세'));await page.locator('.document-result').first().waitFor();
 await page.getByLabel('검색어',{exact:true}).fill('__no_result__');await page.waitForFunction(()=>document.querySelector('[role="status"]')?.textContent.includes('0개'));
 assert.equal(await page.locator('.document-result,.action-result').count(),0);
 await go('/scenes/');assert.equal(await page.locator('.scene-card').count(),meta.scenes.length);await expect(page.locator('main [role=status]')).toContainText(meta.scenes.reduce((n,s)=>n+s.actions.length,0)+'개 행동');
 await page.getByLabel('행동 결정 상태 필터').selectOption('pending');await expect(page.locator('main [role=status]')).toContainText(meta.scenes.reduce((n,s)=>n+s.actions.filter(a=>a.status==='pending').length,0)+'개 행동');
 for(const s of meta.scenes){
  await go('/scenes/'+s.id+'/');assert.equal(await page.locator('.action-row').count(),s.actions.length);
  for(let i=0;i<s.images.length;i++){
   if(s.images.length>1)await page.getByRole('button',{name:/^시안 선택:/}).nth(i).click();
   await page.locator('#scene-image').evaluate(im=>im.decode());assert(await page.locator('#scene-image').evaluate(im=>im.naturalWidth>0));
  }
  if(s.images.length){
  const opener=page.getByRole('button',{name:'시안 크게 보기',exact:true});await opener.click();await page.getByRole('dialog').waitFor();
  await page.keyboard.press('Escape');await expect(page.getByRole('dialog')).toHaveCount(0);await expect(opener).toBeFocused();
  }else{await expect(page.locator('#scene-image')).toHaveCount(0);await expect(page.locator('.scene-no-image')).toBeVisible();}
 }
 await go('/search/?q='+encodeURIComponent('교대'));await page.locator('.action-result').first().waitFor();
 await go('/scenes/battle-focus/#action-swap');await page.locator('.highlight').waitFor();
 console.log(`React navigation, hydration, filters, search, ${meta.scenes.reduce((n,s)=>n+s.actions.length,0)} actions and ${meta.scenes.reduce((n,s)=>n+s.images.length,0)} image views passed.`);
 // Every document is readable without JavaScript and has a real static route.
 for(const p of meta.pages){const response=await fetch(home+p.path);assert.equal(response.status,200,p.path);const body=await response.text();assert(body.includes('id="article-body"'),p.path);assert(body.includes('wiki-boot'),p.path);}
 const nojs=await browser.newContext({javaScriptEnabled:false});const plain=await nojs.newPage();await plain.goto(home+'/wiki/content/abnormalities/yin-yang/fragment/');assert((await plain.locator('#article-body').textContent()).includes('순환 회복'));await nojs.close();
 const bad=await page.evaluate(({docs,base})=>{
  const parsed=Object.fromEntries(Object.entries(docs).map(([id,p])=>[id,new DOMParser().parseFromString(p.html,'text/html')])),errors=[];
  for(const [id,d] of Object.entries(parsed))for(const a of d.querySelectorAll('a[href]')){
   const href=a.getAttribute('href');if(!href.startsWith(base+'/wiki/'))continue;
   const u=new URL(href,'https://example.test'),key=Object.values(docs).find(p=>base+p.path===decodeURIComponent(u.pathname))?.id;
   if(!parsed[key]||u.hash&&!parsed[key].getElementById(decodeURIComponent(u.hash.slice(1))))errors.push(id+': '+href);
  }return errors;
 },{docs,base});assert.deepEqual(bad,[]);
 for(const p of Object.values(docs))for(const u of [...p.html.matchAll(/(?:href|src)="([^"]+)"/g)].map(m=>m[1])){
  if(!u.startsWith(base+'/generated/'))continue;const file=path.join(dist,decodeURIComponent(u.slice(base.length).split('#')[0]));await fs.access(file);
 }
 await go('/screens/');assert.equal(await page.locator('.catalog-row').count(),74);await page.getByLabel('화면 제작 목록 검색').fill('상점');assert(await page.locator('.catalog-row').count()>0&&await page.locator('.catalog-row').count()<74);
 await go('/categories/relics/');await expect(page.locator('.collection-card')).toHaveCount(42);
 await page.getByLabel('유물 검색',{exact:true}).fill('넓어진 지평');await expect(page.locator('.collection-card')).toHaveCount(1);
 await expect(page.locator('.collection-card h2')).toHaveText('넓어진 지평');
 await page.getByLabel('유물 검색',{exact:true}).fill('아직 남은 선물');await expect(page.locator('.collection-card')).toHaveCount(1);
 await expect(page.locator('.collection-card h2')).toHaveText('아직 남은 선물');
 await page.getByLabel('유물 검색',{exact:true}).fill('');await expect(page.locator('.collection-card')).toHaveCount(42);
 // Whole-card preview: title and blank space, native keyboard activation, and independent links.
 const clickableCard=page.locator('.collection-card').filter({has:page.getByRole('heading',{name:'깊은 쐐기',exact:true})});
 const clickableOpener=clickableCard.getByRole('button',{name:'깊은 쐐기 빠르게 보기',exact:true});
 await clickableCard.hover();await expect(clickableCard).toHaveCSS('transform','matrix(1, 0, 0, 1, 0, -3)');await expect(clickableCard).toHaveCSS('border-top-color','rgb(139, 94, 26)');
 await page.screenshot({path:path.join(artifacts,'card-hover-desktop.png'),fullPage:false,animations:'disabled'});
 await page.emulateMedia({reducedMotion:'reduce'});await expect(clickableCard).toHaveCSS('transform','none');await page.emulateMedia({reducedMotion:'no-preference'});
 const summaryBox=await clickableCard.locator('p').nth(1).boundingBox();
 await page.mouse.click(summaryBox.x+15,summaryBox.y+12);await expect(page.getByRole('dialog').locator('#article-body')).toBeVisible();
 assert(page.url().endsWith('/categories/relics/'));await page.keyboard.press('Escape');await expect(clickableOpener).toBeFocused();
 await page.keyboard.press('Enter');await expect(page.getByRole('dialog').locator('#article-body')).toBeVisible();await page.keyboard.press('Escape');
 await expect(clickableOpener).toBeFocused();await page.keyboard.press('Space');await expect(page.getByRole('dialog').locator('#article-body')).toBeVisible();await page.keyboard.press('Escape');
 const headingBox=await clickableCard.getByRole('heading').boundingBox();
 await page.mouse.click(headingBox.x+15,headingBox.y+12);await expect(page.getByRole('dialog').locator('#article-body')).toBeVisible();await page.keyboard.press('Escape');
 await clickableCard.locator('.content-tags').getByRole('link',{name:'#다중타격',exact:true}).click();
 await expect(page.getByRole('dialog')).toHaveCount(0);await expect(page.getByRole('button',{name:'태그: 다중타격',exact:true})).toHaveAttribute('aria-pressed','true');
 await page.getByRole('button',{name:'태그 초기화',exact:true}).click();
 await clickableCard.getByRole('link',{name:'상세 보기',exact:true}).click();await expect(page.locator('#article-body')).toBeVisible();await expect(page.getByRole('dialog')).toHaveCount(0);
 await go('/categories/relics/');await page.mouse.move(0,0);
 const wedge=page.locator('.collection-card').filter({has:page.getByRole('heading',{name:'깊은 쐐기',exact:true})});await expect(wedge).toContainText('2%p');await expect(wedge).toContainText('20%');
 await page.screenshot({path:path.join(artifacts,'relics-cards-desktop.png'),fullPage:true,animations:'disabled'});
 const relicOpener=page.getByRole('button',{name:'깊은 쐐기 빠르게 보기',exact:true});await relicOpener.click();await expect(page.getByRole('dialog').locator('#article-body')).toContainText('같은 시전');
 assert(page.url().endsWith('/categories/relics/'));await page.keyboard.press('Escape');await expect(page.getByRole('dialog')).toHaveCount(0);await expect(relicOpener).toBeFocused();
 const viewControls=page.getByRole('group',{name:'유물 보기 방식'});
 await viewControls.getByRole('button',{name:'비교표',exact:true}).click();await expect(page.locator('.collection-table-row')).toHaveCount(42);
 await page.getByLabel('유물 검색',{exact:true}).fill('2%p');await expect(page.locator('.collection-table-row')).toHaveCount(1);
 await page.getByLabel('유물 구분',{exact:true}).selectOption('전용');await expect(page.locator('.collection-table-row')).toHaveCount(0);await expect(page.getByText('일치하는 항목이 없습니다.',{exact:false})).toBeVisible();
 await page.getByLabel('유물 검색',{exact:true}).fill('');await expect(page.locator('.collection-table-row')).toHaveCount(10);
 await viewControls.getByRole('button',{name:'카드',exact:true}).click();await expect(page.locator('.collection-card')).toHaveCount(10);
 await page.getByLabel('유물 구분',{exact:true}).selectOption('구분 미정');await expect(page.locator('.collection-card')).toHaveCount(4);
 await page.getByLabel('유물 구분',{exact:true}).selectOption('');await viewControls.getByRole('button',{name:'목록',exact:true}).click();await page.reload();await expect(page.locator('.collection-list-row')).toHaveCount(42);
 const borrowed=page.locator('.collection-list-row').filter({hasText:'빌려온 댓가'});await borrowed.locator('summary').first().click();await expect(borrowed.locator('p').first()).toBeVisible();await expect(borrowed).toContainText('10초');
 await viewControls.getByRole('button',{name:'비교표',exact:true}).click();await page.screenshot({path:path.join(artifacts,'relics-table-desktop.png'),fullPage:true,animations:'disabled'});
 await page.setViewportSize({width:390,height:844});assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth+1),false);
 await page.getByRole('button',{name:'깊은 쐐기 빠르게 보기',exact:true}).click();await expect(page.getByRole('dialog').locator('#article-body')).toBeVisible();
 assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth+1),false);await page.keyboard.press('Escape');
 await viewControls.getByRole('button',{name:'카드',exact:true}).click();await page.screenshot({path:path.join(artifacts,'relics-cards-mobile.png'),fullPage:true,animations:'disabled'});
 // Every collection keeps all three views; encyclopedia profiles stay compact without clipping.
 const collections=[...meta.categories.map(c=>({route:'/categories/'+c.id+'/',label:c.title,count:meta.pages.filter(p=>p.category===c.id&&(!p.index||p.id==='content/tags/README')&&(c.id!=='abnormalities'||p.part==='README'||p.id.startsWith('content/tools/'))).length,document:true})),{route:'/screens/',label:'화면',count:74},{route:'/scenes/',label:'장면',count:meta.scenes.length},{route:'/search/?q='+encodeURIComponent('교대'),label:'검색 결과'},...meta.scenes.map(s=>({route:'/scenes/'+s.id+'/',label:'플레이어 행동',count:s.actions.length}))];
 for(const collection of collections){
  await page.setViewportSize({width:1440,height:1100});await go(collection.route);
  const controls=page.getByRole('group',{name:collection.label+' 보기 방식'});await controls.waitFor();
  await controls.getByRole('button',{name:'카드',exact:true}).click();await page.locator('.collection-card').first().waitFor();
  const count=collection.count??await page.locator('.collection-card').count();await expect(page.locator('.collection-card')).toHaveCount(count);await verifyCards();
  if(collection.document){const opener=page.locator('.collection-card button').first();await opener.click();await expect(page.getByRole('dialog').locator('#article-body')).toBeVisible();await page.keyboard.press('Escape');await expect(opener).toBeFocused();}
  await controls.getByRole('button',{name:'비교표',exact:true}).click();await expect(page.locator('.collection-table-row')).toHaveCount(count);
  await controls.getByRole('button',{name:'목록',exact:true}).click();await expect(page.locator('.collection-list-row')).toHaveCount(count);await page.locator('.collection-list-row summary').first().click();
  await controls.getByRole('button',{name:'카드',exact:true}).click();await page.setViewportSize({width:390,height:844});await verifyCards();assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth+1),false,collection.route);
 }
 await page.setViewportSize({width:1440,height:1100});await go('/categories/relics/');await verifyCards();await page.screenshot({path:path.join(artifacts,'relics-uniform-desktop.png'),fullPage:true,animations:'disabled'});
 await go('/categories/fragments/');await verifyCards();await page.screenshot({path:path.join(artifacts,'fragments-cards-desktop.png'),fullPage:true,animations:'disabled'});
 console.log('All 8 document categories, scenes, 8 action lists, screen inventory and search passed all views and compact profile/card checks.');
 // Tags combine across views, preserve URL navigation, and distinguish effects from synergy.
 const tagged=meta.pages.filter(p=>['relics','fragments'].includes(p.category)&&!p.index);
 assert.equal(tagged.length,69);
 assert(tagged.filter(p=>p.category==='relics').every(p=>/^(범용|전용|구분 미정)( ·|$)/.test(p.relic.kind)));
 const withoutEffects=tagged.filter(p=>!p.tags?.length);
 assert.deepEqual(withoutEffects.map(p=>p.id).sort(),['content/abnormalities/big-bad-wolf/fragment','content/abnormalities/funeral-of-dead-butterflies/fragment','content/abnormalities/white-night/fragment']);
 assert(withoutEffects.filter(p=>p.id!=='content/abnormalities/white-night/fragment').every(p=>p.classifications?.includes('파편 없음')));
 assert.deepEqual(withoutEffects.find(p=>p.id==='content/abnormalities/white-night/fragment').classifications,['사도화','원정 누적']);
 const forbiddenTags=['범위증폭','보호막효율증가','무아지경','치명적인군중제어','스택한도해제','공격이어가기','폭발취약','관통피해증가','전용 전환','파편 없음'];
 assert(tagged.every(p=>(p.tags||[]).every(t=>!forbiddenTags.includes(t.label))));
 const namedRelic=id=>meta.pages.find(p=>p.id==='content/relics/'+id);
 assert.equal(namedRelic('widened-horizon').title,'넓어진 지평');
 assert.equal(namedRelic('explosion-area-increase').category,'history');
 assert(!tagged.some(p=>p.id==='content/relics/explosion-area-increase'));
 assert(namedRelic('widened-horizon').relic.summary.includes('4×4'));
 assert.equal(namedRelic('red-thread').title,'아직 남은 선물');
 assert(namedRelic('red-thread').tags.some(t=>t.label==='지속시간'&&t.relation==='effect'));
 assert(namedRelic('unbroken-trance').tags.some(t=>t.label==='시전'&&t.relation==='require'));
 assert(namedRelic('broken-balance').tags.some(t=>t.label==='제어'&&t.relation==='require'));
 assert(namedRelic('broken-balance').tags.some(t=>t.label==='받는피해증가'&&t.relation==='effect'));
 const fragmentTags=name=>meta.pages.find(p=>p.id==='content/abnormalities/'+name+'/fragment').tags;
 assert.deepEqual(fragmentTags('laetitia').filter(t=>t.relation==='effect').map(t=>t.label).sort(),['소환물','피해','폭발','범위'].sort());
 assert(fragmentTags('laetitia').some(t=>t.label==='지속시간'&&t.relation==='synergy'));
 assert.deepEqual(fragmentTags('white-night'),[]);
 assert(!fragmentTags('queen-bee').some(t=>t.label==='소환물'));
 assert(!fragmentTags('clouded-monk').some(t=>t.label==='흡혈'));
 assert(!fragmentTags('yin-yang').some(t=>t.label==='피해'));
 assert(fragmentTags('queen-of-hatred').some(t=>t.label==='보호막'&&t.relation==='synergy'));
 assert(!fragmentTags('queen-of-hatred').some(t=>t.label==='보호막'&&t.relation==='effect'));
 await go('/categories/fragments/');
 await page.getByRole('button',{name:'태그: 소환물',exact:true}).click();
 await page.getByRole('button',{name:'태그: 회복',exact:true}).click();
 await expect(page.locator('.collection-card')).toHaveCount(1);await expect(page.locator('.collection-card h2')).toContainText('음양');
 await verifyCards();await page.screenshot({path:path.join(artifacts,'tags-fragments-desktop.png'),fullPage:true,animations:'disabled'});
 await page.reload();await expect(page.locator('.collection-card')).toHaveCount(1);
 await page.getByLabel('태그 조합 방식').selectOption('any');await expect(page.locator('.collection-card')).not.toHaveCount(1);
 await expect(page.locator('.collection-card h2').filter({hasText:'여왕벌'})).toHaveCount(1);
 await page.goBack();await expect(page.locator('.collection-card')).toHaveCount(1);
 await page.getByRole('button',{name:'태그 초기화',exact:true}).click();await expect(page.locator('.collection-card')).toHaveCount(27);
 // The company-skill proposal is archived and must not appear in current tag results.
 await page.getByRole('button',{name:'태그: 보호막',exact:true}).click();await expect(page.locator('.collection-card')).toHaveCount(3);
 await page.getByLabel('태그 관계').selectOption('synergy');await expect(page.locator('.collection-card')).toHaveCount(2);
 await expect(page.locator('.collection-card h2').filter({hasText:'증오의 여왕'})).toHaveCount(1);
 await page.getByLabel('태그 관계').selectOption('effect');await expect(page.locator('.collection-card')).toHaveCount(2);
 await expect(page.locator('.collection-card h2').filter({hasText:'회사 지급 스킬'})).toHaveCount(0);
 await expect(page.locator('.collection-card h2').filter({hasText:'증오의 여왕'})).toHaveCount(0);
 await page.getByLabel('태그 관계').selectOption('synergy');await expect(page.locator('.collection-card')).toHaveCount(2);
 await go('/categories/relics/?tag='+encodeURIComponent('소환물')+'&tag='+encodeURIComponent('피해'));
 await expect(page.locator('.collection-card')).toHaveCount(3);
 await expect(page.locator('.collection-card').filter({has:page.getByRole('heading',{name:'사냥의 낙인',exact:true})})).toContainText('5초');
 await expect(page.locator('.collection-card').filter({has:page.getByRole('heading',{name:'아직 남은 선물',exact:true})})).toContainText('10초');
 const tagViews=page.getByRole('group',{name:'유물 보기 방식'});
 for(const [view,selector] of [['비교표','.collection-table-row'],['목록','.collection-list-row'],['카드','.collection-card']]){await tagViews.getByRole('button',{name:view,exact:true}).click();await expect(page.locator(selector)).toHaveCount(3);}
 await page.getByLabel('유물 검색',{exact:true}).fill('__empty__');await expect(page.locator('.collection-card')).toHaveCount(0);
 await page.getByLabel('유물 검색',{exact:true}).fill('');await expect(page.locator('.collection-card')).toHaveCount(3);
 await page.getByRole('button',{name:'아직 남은 선물 빠르게 보기',exact:true}).click();const tagDialog=page.getByRole('dialog');
 await tagDialog.getByText('태그의 의미와 적용 근거',{exact:true}).click();await expect(tagDialog).toContainText('유물 대상 태그');
 await page.keyboard.press('Escape');await expect(page.getByRole('dialog')).toHaveCount(0);
 await page.setViewportSize({width:390,height:844});await verifyCards();assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth+1),false);
 await page.screenshot({path:path.join(artifacts,'tags-relics-mobile.png'),fullPage:true,animations:'disabled'});
 await go('/wiki/content/abnormalities/yin-yang/fragment/');
 await page.locator('main .content-tags').getByRole('link',{name:'#회복',exact:true}).click();
 await expect(page.getByRole('button',{name:'태그: 회복',exact:true})).toHaveAttribute('aria-pressed','true');
 await go('/categories/relics/?tag=__unknown__');await expect(page.locator('.collection-card')).toHaveCount(0);
 await page.getByRole('button',{name:'태그 초기화',exact:true}).click();await expect(page.locator('.collection-card')).toHaveCount(42);
 console.log('66 effect-tagged documents and 3 classification-only entries: AND/OR, relationships, URL reload/history, 3 views, preview, source boundaries and mobile passed.');

 await page.setViewportSize({width:390,height:844});await go('/');await page.screenshot({path:path.join(artifacts,'home-mobile.png'),fullPage:true,animations:'disabled'});
 await page.getByRole('button',{name:'탐색 메뉴 열기'}).click();const drawer=page.getByRole('dialog');await drawer.waitFor();await drawer.getByRole('link',{name:'장면·플레이어 행동'}).click();await page.locator('.scene-card').first().waitFor();assert.equal(await page.getByRole('dialog').count(),0);
 await go('/scenes/maintenance/');await page.locator('#scene-image').evaluate(im=>im.decode());await page.screenshot({path:path.join(artifacts,'scene-mobile.png'),fullPage:true,animations:'disabled'});
 const overflow=[];for(const route of ['/','/search/','/scenes/','/screens/','/about/',...meta.categories.map(c=>'/categories/'+c.id+'/'),...meta.scenes.map(s=>'/scenes/'+s.id+'/'),...meta.pages.filter(p=>p.id.startsWith('rules/')||['fragment','equipment'].includes(p.part)).map(p=>p.path)]){
  await go(route);if(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth+1))overflow.push(route);
 }
 assert.deepEqual(overflow,[]);
 assert.deepEqual(errors,[]);assert.deepEqual(consoleErrors,[]);
 // System preference, explicit choice, persistence, and storage-disabled browsers.
 await page.bringToFront();
 await page.emulateMedia({colorScheme:'dark'});
 await expect(page.locator('html')).toHaveAttribute('data-theme','dark');
 await go('/');await page.screenshot({path:path.join(artifacts,'home-dark-mobile.png'),fullPage:true,animations:'disabled'});
 assert.equal(await page.evaluate(()=>getComputedStyle(document.body).backgroundColor),'rgb(14, 13, 11)');
 assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth+1),false);
 await page.emulateMedia({colorScheme:'light'});await expect(page.locator('html')).toHaveAttribute('data-theme','light');
 await page.getByRole('button',{name:'다크 테마',exact:true}).click();
 await expect(page.getByRole('button',{name:'다크 테마',exact:true})).toHaveAttribute('aria-pressed','true');
 await page.setViewportSize({width:1440,height:1100});await go('/');
 await expect(page.locator('html')).toHaveAttribute('data-theme','dark');
 await page.screenshot({path:path.join(artifacts,'home-dark-desktop.png'),fullPage:true,animations:'disabled'});
 await go('/wiki/content/abnormalities/yin-yang/fragment/');await page.reload();
 await expect(page.locator('html')).toHaveAttribute('data-theme','dark');
 await page.screenshot({path:path.join(artifacts,'article-dark-desktop.png'),fullPage:true,animations:'disabled'});
 await page.emulateMedia({media:'print'});assert.equal(await page.evaluate(()=>getComputedStyle(document.body).color),'rgb(29, 27, 23)');await page.emulateMedia({media:'screen'});
 await page.getByRole('button',{name:'다크 테마',exact:true}).click();await page.emulateMedia({colorScheme:'dark'});await page.reload();
 await expect(page.locator('html')).toHaveAttribute('data-theme','light');
 const restricted=await browser.newContext({colorScheme:'dark'});
 await restricted.addInitScript(()=>Object.defineProperty(window,'localStorage',{get(){throw new Error('Storage disabled');}}));
 const privatePage=await restricted.newPage();privatePage.on('pageerror',e=>errors.push(e.message));
 await privatePage.goto(home+'/');await expect(privatePage.getByRole('button',{name:'다크 테마',exact:true})).toHaveAttribute('aria-pressed','true');
 await privatePage.getByRole('button',{name:'다크 테마',exact:true}).click();await expect(privatePage.locator('html')).toHaveAttribute('data-theme','light');await restricted.close();
 assert.deepEqual(errors,[]);assert.deepEqual(consoleErrors,[]);assert.deepEqual(failed,[]);assert.deepEqual(requests,[]);
 const report={status:'PASS',documents:meta.pages.length,actions:meta.scenes.reduce((n,s)=>n+s.actions.length,0),images:meta.scenes.reduce((n,s)=>n+s.images.length,0),staticRoutes:true,readableWithoutJavaScript:true,deepLinks:true,collectionViews:['cards','table','list','persisted','filtered','inline-preview'],collectionCount:collections.length,tags:{pages:66,classificationOnly:3,checks:['AND','OR','relationships','URL','history','three-views','source-boundaries']},cards:{encyclopedia:'content-sized profiles under 330px',otherCollections:448,equalWidths:true,alignedFooters:true},themes:['light','dark','system','persisted','storage-disabled','print'],viewports:['1440×1100','390×844'],externalRequests:requests,errors,consoleErrors,failed,mobileOverflow:overflow};
 await fs.writeFile(path.join(artifacts,'browser-report.json'),JSON.stringify(report,null,2)+'\n');console.log(report);
}finally{await browser?.close();await new Promise(resolve=>server.close(resolve));}
