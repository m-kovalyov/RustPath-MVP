<script setup lang="ts">
import { computed, ref, watch, onBeforeUnmount } from 'vue'
import { useRoute, useRouter, onBeforeRouteLeave, onBeforeRouteUpdate } from 'vue-router'
import { api } from '../api'
import { useLearning } from '../store'
import type { Lesson, Submission } from '../types'
import Icon from './Icon.vue'
import CodeEditor from './CodeEditor.vue'
const route = useRoute(), router = useRouter(), store = useLearning()
const lesson = ref<Lesson | null>(null), code = ref(''), loading = ref(true), error = ref('')
const running = ref(false), result = ref<Submission | null>(null), hintCount = ref(0), saved = ref('')
const index = computed(()=>store.course?.lessons.findIndex(l=>l.id===lesson.value?.id) ?? 0)
const next = computed(()=>store.course?.lessons[index.value+1])
let timer: ReturnType<typeof setTimeout> | undefined
let loadedCode = ''
let generation = 0
let pendingSave: Promise<void> = Promise.resolve()
async function load() {
  const currentGeneration = ++generation
  loading.value=true; error.value=''; result.value=null; lesson.value=null; hintCount.value=0; saved.value=''
  try {
    const id = encodeURIComponent(String(route.params.id))
    const [l,d] = await Promise.all([api<Lesson>(`/lessons/${id}`),api<{code:string|null}>(`/lessons/${id}/draft`)])
    if(currentGeneration!==generation) return
    lesson.value=l; loadedCode=d.code ?? l.starter; code.value=loadedCode
  } catch(e) { if(currentGeneration===generation) error.value=(e as Error).message }
  finally { if(currentGeneration===generation) loading.value=false }
}
watch(()=>route.params.id, load, {immediate:true})
function save(): Promise<void> {
  clearTimeout(timer)
  if(!lesson.value || code.value===loadedCode) return pendingSave
  const id=lesson.value.id, value=code.value
  saved.value='Сохраняем…'
  pendingSave = pendingSave.then(async()=>{
    try { await api(`/lessons/${id}/draft`,{method:'PUT',body:JSON.stringify({code:value})}); if(lesson.value?.id===id && code.value===value) {loadedCode=value; saved.value='Черновик сохранён'} }
    catch { if(lesson.value?.id===id) saved.value='Не удалось сохранить. Скопируйте код перед выходом.' }
  })
  return pendingSave
}
watch(code,()=>{if(!loading.value && code.value!==loadedCode){ result.value=null; clearTimeout(timer); saved.value='Есть изменения'; timer=setTimeout(()=>void save(),800) }})
onBeforeRouteLeave(async()=>{await save(); return !running.value})
onBeforeRouteUpdate(async()=>{await save(); return !running.value})
onBeforeUnmount(()=>{clearTimeout(timer); generation++})
async function run() {
  if(!lesson.value || running.value) return
  running.value=true; error.value=''; result.value=null
  try {
    await save()
    const response = await api<Submission>(`/lessons/${lesson.value.id}/submit`,{method:'POST',body:JSON.stringify({code:code.value})})
    result.value=response; store.progress=response.progress
  } catch(e){ error.value=(e as Error).message }
  finally{running.value=false}
}
function reset() { if(lesson.value && window.confirm('Вернуть исходный код? Текущий черновик будет заменён.')) code.value=lesson.value.starter }
</script>
<template>
  <div class="page lesson-page"><RouterLink to="/" class="back-link">← Учебный маршрут</RouterLink>
    <div v-if="loading" class="lesson-loading" role="status">Открываем уровень…</div>
    <div v-else-if="!lesson" class="note-card" role="alert"><h1>Не удалось открыть уровень</h1><p>{{ error }}</p><button class="secondary" @click="load">Повторить</button></div>
    <template v-else>
      <div class="lesson-header"><div><span class="eyebrow">УРОВЕНЬ {{ index+1 }} / 16 · {{ lesson.minutes }} МИН</span><h1>{{ lesson.title }}</h1><p class="lead">{{ lesson.summary }}</p></div><span class="xp-badge"><Icon name="spark" /> {{ lesson.xp }} XP</span></div>
      <div class="workspace"><section class="theory"><div class="panel-label">01 / РАЗБЕРЁМСЯ</div><article v-for="(section,i) in lesson.theory" :key="i"><h2>{{ section.heading }}</h2><p>{{ section.text }}</p><pre v-if="section.example" class="example"><code>{{ section.example }}</code></pre></article><div class="task-card"><span class="eyebrow">ВАША ЗАДАЧА</span><p>{{ lesson.task }}</p></div><div class="hint-section"><button class="secondary hint-button" :disabled="hintCount>=lesson.hints.length" @click="hintCount++">{{ hintCount>=lesson.hints.length ? 'Все подсказки открыты' : `Подсказка ${hintCount+1} / ${lesson.hints.length}` }} <Icon name="turn" /></button><p v-for="(hint,i) in lesson.hints.slice(0,hintCount)" :key="i" class="hint"><strong>{{ i+1 }}.</strong> {{ hint }}</p></div></section>
        <section class="practice"><div class="editor-bar"><span>02 / ВАША ПРАКТИКА</span><span>solution.rs</span></div><CodeEditor :key="lesson.id" v-model="code" :disabled="running" @run="run"/><div class="editor-controls"><span id="editor-help" class="subtle">Ctrl / ⌘ + Enter — проверить.<br>Esc, затем Tab — выйти из редактора.</span><button class="reset" :disabled="running" @click="reset">Сбросить</button></div><div class="submit-row"><span class="subtle" role="status">{{ saved || 'Изменения сохраняются автоматически' }}</span><button class="primary" :disabled="running || !code.trim()" @click="run">{{ running ? 'Компилируем…' : 'Проверить код' }} <span aria-hidden="true">→</span></button></div>
          <div v-if="error" class="feedback error-feedback" role="alert">{{ error }}</div>
          <div class="tests-panel"><span class="panel-label">ПРОВЕРЯЕМ НЕ ТЕКСТ, А ПОВЕДЕНИЕ</span><ul><li v-for="label in lesson.test_labels" :key="label"><span aria-hidden="true">{{ result?.evaluation.passed ? '✓' : '○' }}</span>{{ label }}</li></ul><p class="subtle">Компиляция Rust + unit-тесты в отдельном контейнере.</p></div>
          <div v-if="running" class="feedback" role="status">Каждая попытка запускается отдельно. Проверка может занять до 30 секунд.</div>
          <div v-if="result" class="feedback" :class="result.evaluation.passed ? 'success-feedback':'error-feedback'" role="status"><h3>{{ result.evaluation.passed ? 'Есть! Ещё один шаг вперёд.' : 'Почти. Давайте разберёмся.' }}</h3><p>{{ result.evaluation.passed ? (result.awarded_xp ? `+${result.awarded_xp} XP. Следующий уровень открыт.` : 'Уровень уже пройден. Отличное повторение!') : 'Посмотрите вывод ниже или откройте подсказку. Опыт не потерян.' }}</p><details :open="!result.evaluation.passed"><summary>Вывод компилятора и тестов · {{ result.evaluation.duration_ms }} мс</summary><pre>{{ result.evaluation.output || 'Нет вывода.' }}</pre></details><button v-if="result.evaluation.passed" class="primary" @click="router.push(next ? `/learn/${next.id}` : '/roadmap')">{{ next ? 'Следующий уровень →' : 'Фундамент готов. Что дальше? →' }}</button></div>
        </section>
      </div>
    </template>
  </div>
</template>
