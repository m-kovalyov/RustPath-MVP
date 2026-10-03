/* Real API/DB UI tests; compiler success is an explicit HTTP fixture, not execution proof. */
const {createRequire}=require('module');
const path=require('path'),fs=require('fs');
const root=path.resolve(__dirname,'..');
const {chromium}=createRequire(path.join(root,'frontend/package.json'))('playwright');
const base=process.env.APP_BASE_URL || 'http://localhost:5173';
const captures=process.env.QA_CAPTURE_DIR;
(async()=>{
 const browser=await chromium.launch({headless:true,...(process.env.BROWSER_EXECUTABLE?{executablePath:process.env.BROWSER_EXECUTABLE}:{}),args:['--no-sandbox']});
 const context=await browser.newContext({viewport:{width:1440,height:1000},colorScheme:'light'});
 const page=await context.newPage();const errors=[],states=[];page.on('pageerror',e=>errors.push(e.message));
 async function snap(name){
  await page.waitForTimeout(300);
  if(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth))throw Error('Horizontal overflow: '+name);
  states.push(name);
  if(captures){fs.mkdirSync(captures,{recursive:true});const html=await page.evaluate(()=>{
   const css=[...document.styleSheets].flatMap(sheet=>{try{return [...sheet.cssRules].map(r=>r.cssText)}catch{return []}}).join('\n');
   const body=document.body.cloneNode(true);
   document.body.querySelectorAll('input').forEach((input,i)=>{const copy=body.querySelectorAll('input')[i];copy.setAttribute('value',input.value);if(input.checked)copy.setAttribute('checked','');else copy.removeAttribute('checked')});
   return '<!doctype html><html lang="ru"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><style>'+css+'</style></head><body>'+body.innerHTML.replace(/<script\b[^>]*>[\s\S]*?<\/script>/gi,'')+'</body></html>'
  });fs.writeFileSync(path.join(captures,name+'.html'),html)}
 }
 await page.goto(base);await page.getByRole('heading',{name:'Код, который вы понимаете.'}).waitFor();await snap('home-desktop');
 // Compact accordion and search work without downloading every lesson.
 await page.getByRole('searchbox',{name:'Найти тему'}).fill('не существующая тема');await page.getByText('Такой темы здесь нет').waitFor();await snap('search-empty');
 await page.getByRole('button',{name:'Сбросить поиск'}).click();
 await page.getByRole('button',{name:/Продвинутый Rust/}).click();await page.getByText('Контракты и обобщения',{exact:true}).waitFor();await snap('advanced-desktop');
 await page.getByRole('button',{name:/С нуля/}).click();await page.getByRole('link',{name:'Первый урок'}).click();await page.getByRole('heading',{name:'Знакомство с Rust',exact:true}).waitFor();await snap('intro-theory');
 await page.getByRole('button',{name:'Сохранить тему'}).click();await page.getByRole('button',{name:'Тема сохранена'}).waitFor();
 await page.getByRole('button',{name:'Попробовать самому'}).click();await snap('quiz-desktop');
 await page.getByRole('radio',{name:'Программа считает неверную скидку, но типы совпадают'}).check();await page.getByRole('button',{name:'Проверить ответ'}).click();await page.getByText('Разберёмся и попробуем снова.').waitFor();
 const wrong=await page.evaluate(async()=>await(await fetch('/api/progress')).json());if(wrong.xp!==0)throw Error('Wrong quiz awarded XP');
 await page.getByRole('radio',{name:'Функция возвращает строку, хотя обещает число'}).check();await page.getByRole('button',{name:'Проверить ответ'}).click();await page.getByText('Поняли. Сделали. Идём дальше.').waitFor();await snap('quiz-success');
 await page.reload();await page.getByRole('heading',{name:'Знакомство с Rust',exact:true}).waitFor();await page.getByRole('button',{name:'Тема сохранена'}).waitFor();
 const savedProgress=await page.evaluate(async()=>await(await fetch('/api/progress')).json());if(savedProgress.xp!==20 || !savedProgress.completed.includes('intro-rust'))throw Error('Real quiz persistence failed');
 await page.getByRole('link',{name:'Повторение',exact:true}).click();await page.getByRole('heading',{name:'Вернуться. Понять глубже.'}).waitFor();await page.getByRole('heading',{name:'Знакомство с Rust',exact:true}).waitFor();await snap('review-bookmarks');
 await page.getByRole('button',{name:'Убрать',exact:true}).click();await page.getByRole('heading',{name:'Пока без закладок'}).waitFor();await snap('review-empty');
 await page.goto(base+'/learn/adv-min');await page.getByRole('heading',{name:'Одна функция для разных типов'}).waitFor();await page.getByRole('tab',{name:'Материалы'}).click();await snap('materials');
 await page.getByRole('tab',{name:'02 · Практика'}).click();await page.waitForSelector('.cm-content');await snap('code-practice');
 const expected='pub fn minimum<T: Ord + Copy>(a: T, b: T) -> T {\n    if a <= b { a } else { b }\n}';
 await page.locator('.cm-content').click();await page.keyboard.press('Control+a');await page.keyboard.insertText(expected);await page.getByText('Черновик сохранён',{exact:true}).waitFor();
 await page.reload();await page.getByRole('tab',{name:'02 · Практика'}).click();await page.waitForSelector('.cm-content');if(await page.locator('.cm-content').innerText()!==expected)throw Error('Draft failed after reload');
 // Prevent dependence on a Docker host in UI contract tests. Test compiler-error and success surfaces explicitly.
 await page.route('**/api/lessons/adv-min/submit',r=>r.fulfill({json:{evaluation:{passed:false,output:'error[E0308]: mismatched types\nUI contract fixture',duration_ms:10},awarded_xp:0,progress:savedProgress}}));
 await page.getByRole('button',{name:'Проверить код'}).click();await page.getByText('Разберёмся и попробуем снова.').waitFor();await snap('code-error-fixture');await page.unroute('**/api/lessons/adv-min/submit');
 await page.route('**/api/lessons/adv-min/submit',r=>r.fulfill({json:{evaluation:{passed:true,output:'test result: ok. (UI fixture)',duration_ms:10},awarded_xp:75,progress:{...savedProgress,xp:95,completed:['intro-rust','adv-min']}}}));
 await page.getByRole('button',{name:'Проверить код'}).click();await page.getByText('Поняли. Сделали. Идём дальше.').waitFor();await snap('code-success-fixture');
 await page.goto(base+'/lab');await page.getByRole('heading',{name:'Кто владеет данными сейчас?'}).waitFor();await snap('lab-create');await page.getByRole('button',{name:'Следующий шаг'}).click();await snap('lab-move');await page.getByRole('button',{name:'Следующий шаг'}).click();await snap('lab-borrow');await page.getByRole('button',{name:'Следующий шаг'}).click();await snap('lab-drop');
 await page.emulateMedia({reducedMotion:'reduce'});if(await page.locator('.data-node').evaluate(e=>getComputedStyle(e).transform)!=='none')throw Error('Reduced motion not respected');await page.emulateMedia({reducedMotion:'no-preference'});
 await page.goto(base+'/roadmap');await page.getByRole('heading',{name:'Уроки — начало. Проект — проверка.'}).waitFor();await snap('roadmap-desktop');
 await page.setViewportSize({width:390,height:844});await page.goto(base);await page.getByRole('heading',{name:'Код, который вы понимаете.'}).waitFor();await page.getByRole('button',{name:/С нуля/}).click();await snap('home-mobile');
 await page.goto(base+'/learn/intro-rust');await page.getByRole('button',{name:'Попробовать самому'}).click();await snap('quiz-mobile');
 await page.goto(base+'/learn/adv-min');await page.getByRole('tab',{name:'02 · Практика'}).click();await page.waitForSelector('.cm-content');await snap('code-mobile');
 await page.goto(base+'/lab');await page.getByRole('button',{name:'Следующий шаг'}).click();await snap('lab-mobile');
 await page.goto(base+'/review');await page.getByRole('heading',{name:'Вернуться. Понять глубже.'}).waitFor();await snap('review-mobile');
 await page.emulateMedia({colorScheme:'dark'});await page.setViewportSize({width:1440,height:1000});await page.goto(base);await page.getByRole('heading',{name:'Код, который вы понимаете.'}).waitFor();await snap('home-dark');
 if(captures){const file=path.join(captures,'home-dark.html');fs.writeFileSync(file,fs.readFileSync(file,'utf8').replace(/@media\s*\(prefers-color-scheme:\s*dark\)/g,'@media all'))}
 await page.emulateMedia({colorScheme:'light'});await page.route('**/api/session',r=>r.fulfill({status:503,json:{error:'Сервис временно недоступен.'}}));await page.reload();await page.getByRole('heading',{name:'Вернёмся к обучению?'}).waitFor();await snap('connection-error');
 if(errors.length)throw Error(errors.join('\n'));
 fs.writeFileSync(path.join(root,'docs/ui-qa.json'),JSON.stringify({version:'0.2.0',states,widths:[1440,390],page_errors:errors,horizontal_overflow:false,quiz_and_bookmark_persistence:'real API/PostgreSQL',code_draft_persistence:'real API/PostgreSQL',compiler_states:'explicit fixtures, not sandbox execution',reduced_motion:'pass'},null,2));
 console.log(states.length+' UI states PASS; real quiz/bookmark/draft persistence; compiler responses are fixtures.');
 await browser.close();process.exit(0);
})().catch(e=>{console.error(e);process.exit(1)});
