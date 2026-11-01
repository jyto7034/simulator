// Functional offline browser QA. Uses the repository's existing Playwright install.
const path=require('path'),fs=require('fs'),assert=require('node:assert/strict');
const {pathToFileURL}=require('url');
const root=path.resolve(__dirname,'../../..');
const {chromium}=require(path.join(root,'content_assembler/node_modules/playwright'));
const out=process.env.WIKI_QA_OUTPUT||'/tmp/ark-unified-wiki-20261018';
fs.mkdirSync(out,{recursive:true});
(async()=>{
 const browser=await chromium.launch({headless:true,args:['--no-sandbox']});
 try{
 const page=await browser.newPage({viewport:{width:1440,height:1100},deviceScaleFactor:1});
 const errors=[],failed=[],remote=[];
 page.on('pageerror',e=>errors.push(e.message));page.on('requestfailed',r=>failed.push(r.url()));
 page.on('request',r=>{if(/^https?:/.test(r.url()))remote.push(r.url())});
 const base=pathToFileURL(path.join(root,'core_enhanced/docs/wiki/index.html')).href;
 await page.goto(base);await page.locator('.directory').waitFor();await page.evaluate(()=>document.fonts.ready);
 const data=await page.evaluate(()=>JSON.parse(document.querySelector('#wiki-data').textContent));
 if(process.argv.includes('--integration-only')){
  const oldGuide=new URL(data.uiOriginal,base).href;
  await page.goto(oldGuide);await page.getByRole('link',{name:'↗ 통합 게임 위키',exact:true}).click();
  await page.locator('.directory').waitFor();assert.equal(page.url(),base);
  await page.evaluate(()=>location.hash='page=ui%2Findex');await page.locator('#article-body').waitFor();
  await page.locator('#article-body').getByRole('link',{name:'ARK 통합 게임 위키 열기',exact:true}).click();
  await page.locator('.directory').waitFor();assert.deepEqual(errors,[]);assert.deepEqual(failed,[]);assert.deepEqual(remote,[]);
  console.log('PASS: original UI HTML → unified wiki; embedded UI index → unified home.');return;
 }
 const go=async hash=>{await page.evaluate(hash=>location.hash=hash,hash);await page.waitForTimeout(25)};
 await page.locator('.herofigure img').evaluate(im=>im.decode());
 await page.screenshot({path:path.join(out,'home-desktop.png'),fullPage:true,animations:'disabled'});
 await go('category=abnormalities');assert.equal(await page.locator('.listrow').count(),37);
 await page.selectOption('#tier','최종 보스');assert.equal(await page.locator('.listrow').count(),4);
 await page.selectOption('#tier','');await page.fill('#list-search','음양');assert.equal(await page.locator('.listrow').count(),1);
 await page.locator('.listrow').click();await page.locator('#article-body').waitFor();
 await page.getByRole('link',{name:'스킬 파편',exact:true}).last().click();await page.waitForTimeout(30);
 assert((await page.locator('#article-body').textContent()).includes('순환 회복'));
 await page.screenshot({path:path.join(out,'article-desktop.png'),fullPage:true,animations:'disabled'});
 await page.fill('#search','교대');await page.waitForTimeout(200);
 assert(await page.locator('.action-result').count()>0);await page.locator('.action-result').first().click();await page.locator('.highlight').waitFor();
 assert(await page.locator('.highlight').isVisible());await page.locator('.highlight a').click();await page.locator('#article-body').waitFor();
 assert(await page.locator('#article-body').isVisible());
 await page.fill('#search','__nothing_matches__');await page.waitForTimeout(200);assert.equal(await page.locator('#results .listrow').count(),0);
 await page.fill('#search','깨진 맹세');await page.waitForTimeout(200);const current=await page.locator('#results .listrow').count();
 assert.equal(current,0);await page.getByRole('link',{name:'아카이브에서 검색',exact:true}).click();await page.waitForTimeout(100);assert((await page.locator('h1').textContent()).includes('아카이브 검색'));assert(await page.locator('#results .listrow').count()>0);
 await go('scenes');assert.equal(await page.locator('.scene-card').count(),data.scenes.length);assert.equal(await page.locator('.action-link').count(),data.scenes.reduce((n,s)=>n+s.actions.length,0));
 await page.selectOption('#scene-status','pending');assert.equal(await page.locator('.action-link').count(),data.scenes.reduce((n,s)=>n+s.actions.filter(a=>a.status==='pending').length,0));
 await page.selectOption('#scene-status','');
 for(const s of data.scenes){
  await go('scene='+s.id);assert.equal(await page.locator('.action-table tbody tr').count(),s.actions.length);
  for(let i=0;i<s.images.length;i++){
   if(s.images.length>1)await page.click(`[data-image="${i}"]`);
   await page.locator('#scene-image').evaluate(im=>im.decode());assert(await page.locator('#scene-image').evaluate(im=>im.naturalWidth>0));
  }
  if(s.images.length){await page.click('#enlarge');assert(await page.locator('#lightbox').isVisible());await page.keyboard.press('Escape');assert(!await page.locator('#lightbox').isVisible());}
  else {assert.equal(await page.locator('#scene-image').count(),0);assert(await page.locator('.scene-no-image').isVisible());}
 }
 const badAnchors=await page.evaluate(data=>{
  const bad=[];const parsed=Object.fromEntries(Object.entries(data.pages).map(([k,p])=>[k,new DOMParser().parseFromString(p.html,'text/html')]));
  for(const [id,doc] of Object.entries(parsed))for(const a of doc.querySelectorAll('a[href^="#page="]')){
   const p=new URLSearchParams(a.getAttribute('href').slice(1)),target=parsed[p.get('page')];
   if(!target||p.get('section')&&!target.getElementById(p.get('section')))bad.push(id+': '+a.getAttribute('href'));
  }
  return bad;
 },data);assert.deepEqual(badAnchors,[]);
 for(const id of Object.keys(data.pages)){
  await go('page='+encodeURIComponent(id));assert.equal(await page.locator('#article-body').count(),1);
  assert(!(await page.locator('#article-body').textContent()).includes('WIKITOKEN'),'unrendered inline token '+id);
 }
 console.log(`Desktop, separate current/archive search, all documents, action links and ${data.scenes.reduce((n,s)=>n+s.images.length,0)} images passed.`);
 await go('catalog');assert.equal(await page.locator('.catalog-row').count(),74);
 await page.fill('#catalog-search','상점');assert(await page.locator('.catalog-row').count()>0);assert(await page.locator('.catalog-row').count()<74);
 await page.locator('.catalog-row summary').first().click();assert.equal(await page.locator('.catalog-row[open]').count(),1);
 await page.setViewportSize({width:390,height:844});await go('home');
 await page.screenshot({path:path.join(out,'home-mobile.png'),fullPage:true,animations:'disabled'});
 assert(await page.locator('#sidebar').evaluate(el=>el.inert));await page.click('#menu');assert(await page.locator('#sidebar').evaluate(el=>!el.inert));
 await page.locator('[data-nav="scenes"]').click();await page.locator('.scenegrid').waitFor();assert(await page.locator('#sidebar').evaluate(el=>el.inert));
 await go('scene=maintenance');await page.locator('#scene-image').evaluate(im=>im.decode());
 await page.screenshot({path:path.join(out,'scene-mobile.png'),fullPage:true,animations:'disabled'});
 const overflow=[];
 for(const hash of ['home','scenes','catalog','about',...data.categories.map(c=>'category='+c.id),...data.scenes.map(s=>'scene='+s.id),...Object.keys(data.pages).map(id=>'page='+encodeURIComponent(id))]){
  await go(hash);if(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth+1))overflow.push(hash);
 }
 assert.deepEqual(overflow,[]);assert.deepEqual(errors,[]);assert.deepEqual(failed,[]);assert.deepEqual(remote,[]);
 const result={status:'PASS',pages:Object.keys(data.pages).length,actions:data.scenes.reduce((n,s)=>n+s.actions.length,0),images:data.scenes.reduce((n,s)=>n+s.images.length,0),viewports:['1440×1100','390×844'],offline:true,errors,failed,remote,mobileOverflow:overflow};
 fs.writeFileSync(path.join(out,'browser-report.json'),JSON.stringify(result,null,2)+'\n');console.log(result);
 }finally{await browser.close()}
})().catch(e=>{console.error(e);process.exit(1)});
