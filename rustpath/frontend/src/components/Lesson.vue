<script setup lang="ts">
import { computed, ref, watch, onBeforeUnmount, defineAsyncComponent } from 'vue'
import { useRoute, useRouter, onBeforeRouteLeave, onBeforeRouteUpdate } from 'vue-router'
import { api } from '../api'
import { useLearning } from '../store'
import { nextInTrack } from '../progression'
import type { Lesson, Submission } from '../types'
import Icon from './Icon.vue'
const CodeEditor=defineAsyncComponent(()=>import('./CodeEditor.vue'))
import OwnershipLab from './OwnershipLab.vue'
const route=useRoute(),router=useRouter(),store=useLearning()
const lesson=ref<Lesson|null>(null),code=ref(''),loading=ref(true),error=ref(''),tab=ref('understand')
const running=ref(false),result=ref<Submission|null>(null),hintCount=ref(0),saved=ref(''),option=ref<number|null>(null)
const trackLessons=computed(()=>store.course?.lessons.filter(l=>l.track===lesson.value?.track) ?? [])
const index=computed(()=>trackLessons.value.findIndex(l=>l.id===lesson.value?.id))
const next=computed(()=>nextInTrack(store.course?.lessons ?? [],lesson.value?.id ?? ''))
const track=computed(()=>store.course?.tracks.find(t=>t.id===lesson.value?.track))
const diagnostic=computed(()=>{
 const out=result.value?.evaluation.output ?? ''
 if(out.includes('E0384'))return 'Переменная неизменяема. Проверьте объявление let и необходимость mut.'
 if(out.includes('E0308'))return 'Типы не совпали. Сравните результат выражения с типом в сигнатуре. Проверьте точку с запятой.'
 if(out.includes('E0382'))return 'Значение уже перемещено. Проверьте, не нужен ли здесь заимствованный доступ через &.'
 if(out.includes('E0597')||out.includes('E0515'))return 'Ссылка пережила данные или указывает на локальное значение. Пересмотрите владение результатом.'
 return 'Сопоставьте условие с выводом тестов. Пустой ввод и границы часто помогают найти ошибку.'
})
let timer:ReturnType<typeof setTimeout>|undefined,loadedCode='',generation=0
let pendingSave:Promise<boolean>=Promise.resolve(true)
async function load(){
 const current=++generation;loading.value=true;error.value='';result.value=null;lesson.value=null;hintCount.value=0;saved.value='';option.value=null;tab.value='understand'
 try{const id=encodeURIComponent(String(route.params.id));const [l,d]=await Promise.all([api<Lesson>(`/lessons/${id}`),api<{code:string|null}>(`/lessons/${id}/draft`)]);if(current!==generation)return;lesson.value=l;loadedCode=d.code ?? l.starter;code.value=loadedCode;store.selectTrack(l.track)}catch(e){if(current===generation)error.value=(e as Error).message}finally{if(current===generation)loading.value=false}
}
watch(()=>route.params.id,load,{immediate:true})
function save():Promise<boolean>{
 clearTimeout(timer)
 if(!lesson.value || lesson.value.kind==='quiz' || code.value===loadedCode)return pendingSave
 const id=lesson.value.id,value=code.value;saved.value='Сохраняем…'
 pendingSave=pendingSave.then(async()=>{try{await api(`/lessons/${id}/draft`,{method:'PUT',body:JSON.stringify({code:value})});if(lesson.value?.id===id && code.value===value){loadedCode=value;saved.value='Черновик сохранён'}return true}catch{if(lesson.value?.id===id)saved.value='Не удалось сохранить. Скопируйте код.';return false}})
 return pendingSave
}
watch(code,()=>{if(!loading.value && code.value!==loadedCode){result.value=null;clearTimeout(timer);saved.value='Есть изменения';timer=setTimeout(()=>void save(),800)}})
watch(option,()=>{result.value=null})
async function beforeLeave(){if(running.value)return false;const ok=await save();return ok || window.confirm('Черновик не удалось сохранить. Выйти и потерять последние изменения?')}
onBeforeRouteLeave(beforeLeave);onBeforeRouteUpdate(beforeLeave)
onBeforeUnmount(()=>{clearTimeout(timer);generation++})
async function run(){
 if(!lesson.value || running.value || (lesson.value.kind==='quiz' && option.value===null))return
 running.value=true;error.value='';result.value=null
 try{if(lesson.value.kind==='code')await save();const quiz=lesson.value.kind==='quiz';const response=await api<Submission>(`/lessons/${lesson.value.id}/${quiz?'answer':'submit'}`,{method:'POST',body:JSON.stringify(quiz?{option:option.value}:{code:code.value})});result.value=response;store.progress=response.progress}catch(e){error.value=(e as Error).message}finally{running.value=false}
}
async function bookmark(){if(!lesson.value)return;error.value='';try{await store.toggleBookmark(lesson.value.id)}catch(e){error.value=(e as Error).message}}
function tabKeys(event:KeyboardEvent){
 const ids=['understand','practice','read'];let i=ids.indexOf(tab.value);
 if(event.key==='ArrowRight')i=(i+1)%3;else if(event.key==='ArrowLeft')i=(i+2)%3;else if(event.key==='Home')i=0;else if(event.key==='End')i=2;else return;
 event.preventDefault();tab.value=ids[i];document.getElementById(`tab-${tab.value}`)?.focus()
}
function reset(){if(lesson.value && window.confirm('Вернуть исходный код? Черновик будет заменён.'))code.value=lesson.value.starter}
</script>
<template><div class="page lesson-page">
 <RouterLink to="/" class="back-link">← {{ track?.title ?? 'Курс' }}</RouterLink>
 <div v-if="loading" class="lesson-loading" role="status">Открываем урок…</div>
 <div v-else-if="!lesson" class="empty-state" role="alert"><h1>Урок пока недоступен</h1><p>{{ error }}</p><RouterLink to="/" class="primary">Вернуться к курсу</RouterLink></div>
 <template v-else><div class="lesson-header"><div><span class="eyebrow">{{ track?.title }} / УРОК {{ index+1 }} ИЗ {{ trackLessons.length }}</span><h1>{{ lesson.title }}</h1><p class="lead">{{ lesson.summary }}</p></div><div class="lesson-tools"><span class="xp-badge"><Icon name="spark"/>{{ lesson.xp }} XP · {{ lesson.minutes }} мин</span><button class="secondary bookmark-button" :aria-pressed="store.bookmarks.includes(lesson.id)" :disabled="store.bookmarkBusy" @click="bookmark">{{ store.bookmarks.includes(lesson.id)?'Тема сохранена':'Сохранить тему' }}</button></div></div>
 <div class="lesson-tabs" role="tablist" aria-label="Этап урока" @keydown="tabKeys"><button v-for="item in [{id:'understand',label:'01 · Разобраться'},{id:'practice',label:'02 · Практика'},{id:'read',label:'Материалы'}]" :key="item.id" :id="`tab-${item.id}`" role="tab" :tabindex="tab===item.id?0:-1" :aria-selected="tab===item.id" aria-controls="lesson-panel" :class="{selected:tab===item.id}" @click="tab=item.id">{{ item.label }}</button></div>
 <p v-if="error" class="feedback error-feedback" role="alert">{{ error }}</p>
 <section id="lesson-panel" role="tabpanel" :aria-labelledby="`tab-${tab}`">
 <div v-if="tab==='understand'" class="reading-layout"><div class="reading-column"><p class="learning-outcome"><strong>Вы научитесь:</strong> {{ lesson.outcome }}</p><article v-for="(section,i) in lesson.theory" :key="i"><h2>{{ section.heading }}</h2><p>{{ section.text }}</p><pre v-if="section.example" class="example"><code>{{ section.example }}</code></pre></article><OwnershipLab v-if="lesson.visual==='ownership'"/><aside class="mistake-note"><span class="eyebrow">ЧТО НЕ ПЕРЕПУТАТЬ</span><p>{{ lesson.common_mistake }}</p></aside><button class="primary" @click="tab='practice'">Попробовать самому →</button></div><aside class="lesson-guide"><span class="eyebrow">ОДИН УРОК — ОДНА ИДЕЯ</span><h3>Не торопитесь.</h3><p>Прочитайте пример. Предскажите результат. Затем проверьте свою идею на практике.</p><p class="subtle">{{ lesson.kind==='quiz'?'Этот урок — вопрос на понимание. Компиляция не нужна.':'Тесты проверяют поведение. Следуйте также учебному приёму из условия.' }}</p></aside></div>
 <div v-else-if="tab==='practice'" class="practice-layout"><aside class="practice-brief"><span class="eyebrow">ВАША ЗАДАЧА</span><h2>Закрепим идею.</h2><p>{{ lesson.task }}</p><button class="text-button" @click="tab='understand'">Вернуться к объяснению</button><div class="hint-section"><button class="secondary hint-button" :disabled="hintCount>=lesson.hints.length" @click="hintCount++">{{ hintCount>=lesson.hints.length?'Все подсказки открыты':`Подсказка ${hintCount+1} / ${lesson.hints.length}` }}<Icon name="turn"/></button><p v-for="(hint,i) in lesson.hints.slice(0,hintCount)" :key="i" class="hint">{{ i+1 }}. {{ hint }}</p></div></aside><div class="practice">
 <fieldset v-if="lesson.kind==='quiz' && lesson.quiz" class="quiz-card"><legend>{{ lesson.quiz.question }}</legend><label v-for="(answer,i) in lesson.quiz.options" :key="i" class="quiz-option" :class="{selected:option===i}"><input type="radio" name="answer" :value="i" v-model="option" :disabled="running"><span>{{ answer }}</span></label><button class="primary" :disabled="running || option===null" @click="run">{{ running?'Проверяем…':'Проверить ответ →' }}</button></fieldset>
 <template v-else><div class="editor-bar"><span>solution.rs</span><span>Rust / edition 2024</span></div><CodeEditor :key="lesson.id" v-model="code" :disabled="running" @run="run"/><div class="editor-controls"><span id="editor-help" class="subtle">Ctrl / ⌘ + Enter — проверить.<br>Esc, Tab — выйти из редактора.</span><button class="reset" :disabled="running" @click="reset">Сбросить</button></div><div class="submit-row"><span class="subtle" role="status">{{ saved || 'Черновик сохраняется автоматически' }}</span><button class="primary" :disabled="running || !code.trim()" @click="run">{{ running?'Компилируем…':'Проверить код →' }}</button></div><details class="test-details"><summary>Что проверяют тесты · {{ lesson.test_labels.length }}</summary><ul><li v-for="label in lesson.test_labels" :key="label">{{ label }}</li></ul><p class="subtle">Только стандартная библиотека Rust. Проверка в отдельном контейнере.</p></details></template>
 <p v-if="running" class="feedback" role="status">{{ lesson.kind==='quiz'?'Проверяем ответ…':'Компиляция и тесты могут занять до 30 секунд.' }}</p><div v-if="result" :class="['feedback',result.evaluation.passed?'success-feedback':'error-feedback']" role="status"><h3>{{ result.evaluation.passed?'Поняли. Сделали. Идём дальше.':'Разберёмся и попробуем снова.' }}</h3><p v-if="lesson.kind==='quiz'">{{ result.evaluation.output }}</p><p v-else-if="!result.evaluation.passed">{{ diagnostic }}</p><p v-if="result.evaluation.passed">{{ result.awarded_xp?`+${result.awarded_xp} XP.`:'Урок уже пройден — это полезное повторение.' }}</p><details v-if="lesson.kind==='code'" :open="!result.evaluation.passed"><summary>Вывод компилятора и тестов · {{ result.evaluation.duration_ms }} мс</summary><pre>{{ result.evaluation.output || 'Нет вывода.' }}</pre></details><button v-if="result.evaluation.passed" class="primary" @click="router.push(next?`/learn/${next.id}`:'/roadmap')">{{ next?'Следующий урок →':'Маршрут пройден. Что дальше? →' }}</button></div>
 </div></div>
 <div v-else class="reading-column materials"><h2>Если хотите понять глубже</h2><p>Пособия дополняют этот урок. Задания RustPath написаны отдельно; сайт не является официальным курсом этих университетов.</p><a v-for="reading in lesson.readings" :key="reading.url" :href="reading.url" target="_blank" rel="noopener noreferrer" class="reading-link"><span>{{ reading.title }}</span><span aria-hidden="true">↗</span></a><p class="subtle">Некоторые материалы — на английском. Университетские курсы могут иметь собственные предварительные требования.</p></div>
 </section></template></div></template>
