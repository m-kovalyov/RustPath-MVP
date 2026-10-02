<script setup lang="ts">
import Icon from './Icon.vue'
import { computed } from 'vue'
import { useLearning } from '../store'
const store = useLearning()
const currentModule = computed(() => store.course?.modules.find(m => m.id === store.next?.module))
const lessonsIn = (id: string) => store.course?.lessons.filter(l => l.module === id) ?? []
const doneIn = (id: string) => lessonsIn(id).filter(l => store.progress.completed.includes(l.id)).length
</script>
<template>
  <div class="page home">
    <section class="hero">
      <div><span class="eyebrow">ОТ ПЕРВОЙ СТРОКИ — К СВОИМ ПРОЕКТАМ</span><h1>Rust сложный.<br><span class="text-blue">Ваш путь — нет.</span></h1><p class="lead">Одна идея. Немного практики. Маленькая победа.<br class="desktop"> Учитесь писать надёжный код, шаг за шагом.</p>
        <RouterLink v-if="store.next" :to="`/learn/${store.next.id}`" class="primary hero-button">{{ store.progress.completed.length ? 'Продолжить обучение' : 'Начать бесплатно' }} <span aria-hidden="true">↗</span></RouterLink>
        <RouterLink v-else to="/roadmap" class="primary hero-button">Основы пройдены. Что дальше? ↗</RouterLink>
        <span class="hero-caption">16 практических уровней · на русском · без установки Rust</span>
      </div>
      <div class="hero-card" aria-label="Пример Rust-кода"><div class="code-title"><span class="live-dot"></span> маленькое начало <span>main.rs</span></div><pre><span class="code-purple">fn</span> <span class="code-blue">main</span>() {
    <span class="code-purple">let</span> you = <span class="code-green">"разработчик"</span>;
    <span class="code-purple">let mut</span> skills = <span class="code-orange">0</span>;

    <span class="code-muted">// Каждая попытка — шаг вперёд.</span>
    skills += <span class="code-orange">1</span>;
    println!(<span class="code-green">"Привет, {you}! {skills}"</span>);
}</pre><div class="code-footer"><span>●</span> Собираем уверенность, не только код</div></div>
    </section>
    <section class="stats" aria-label="Ваш прогресс"><div><span class="stat-label">Пройдено уровней</span><strong>{{ store.progress.completed.length }} <small>/ 16</small></strong></div><div><span class="stat-label">Опыт за практику</span><strong>{{ store.progress.xp }} <small>XP</small></strong></div><div><span class="stat-label">Дней подряд <small>(UTC)</small></span><strong>{{ store.progress.streak }} <small>дн.</small></strong></div><div class="stat-progress"><span class="stat-label">Фундамент Rust</span><strong>{{ store.percent }}<small>%</small></strong><div class="progress-track" role="progressbar" :aria-valuenow="store.percent" aria-valuemin="0" aria-valuemax="100" aria-label="Фундамент Rust"><span :style="{width:`${store.percent}%`}"></span></div></div></section>
    <div class="course-layout">
      <section class="modules"><div class="section-heading"><div><span class="eyebrow">УЧЕБНЫЙ МАРШРУТ</span><h2>Всё начинается с основ.</h2></div><span class="subtle">4 модуля / 16 уровней</span></div>
        <article v-for="module in store.course?.modules" :key="module.id" class="module" :class="{'current-module':currentModule?.id===module.id}"><div class="module-heading"><span class="module-number">{{ String(module.number).padStart(2,'0') }}</span><div><h3>{{ module.title }}</h3><p>{{ module.description }}</p></div><span class="module-count">{{ doneIn(module.id) }}/4</span></div>
          <ol class="lesson-list"><li v-for="(lesson,index) in lessonsIn(module.id)" :key="lesson.id"><RouterLink v-if="store.isUnlocked(lesson.id)" :to="`/learn/${lesson.id}`" class="lesson-row" :class="{'lesson-current':store.next?.id===lesson.id}"><span class="lesson-indicator" :class="{'complete':store.progress.completed.includes(lesson.id)}">{{ store.progress.completed.includes(lesson.id) ? '✓' : index+1 }}</span><span class="lesson-title">{{ lesson.title }}<small>{{ lesson.minutes }} мин · {{ lesson.xp }} XP</small></span><span class="lesson-action">{{ store.progress.completed.includes(lesson.id) ? 'Повторить' : 'Начать' }} <span aria-hidden="true">→</span></span></RouterLink><div v-else class="lesson-row locked" :aria-label="`${lesson.title}: сначала завершите предыдущий уровень`"><span class="lesson-indicator">{{ index+1 }}</span><span class="lesson-title">{{ lesson.title }}<small>{{ lesson.minutes }} мин · {{ lesson.xp }} XP</small></span><span class="lock-label">Закрыт</span></div></li></ol>
        </article>
      </section>
      <aside class="sidebar"><div class="note-card blue-note"><span class="eyebrow">ВАШ СЛЕДУЮЩИЙ ШАГ</span><h3>{{ store.next?.title ?? 'Фундамент готов!' }}</h3><p>{{ store.next?.summary ?? 'Переходите к проектам и углублённым темам из маршрута.' }}</p><RouterLink v-if="store.next" :to="`/learn/${store.next.id}`" class="text-link">Открыть уровень →</RouterLink><RouterLink v-else to="/roadmap" class="text-link">Посмотреть маршрут →</RouterLink></div><div class="note-card"><Icon name="turn" :size="32" class="note-icon" /><h3>Ошибаться — часть пути.</h3><p>Компилятор не ставит оценки. Он помогает понять, что изменить. Подсказки всегда рядом.</p></div><div class="note-card"><span class="eyebrow">ПОСЛЕ ОСНОВ</span><h3>Не только синтаксис.</h3><p>Дальше — async, backend, базы данных и собственный сервис. Продвинутые модули пока в плане.</p><RouterLink to="/roadmap" class="text-link">Весь путь к практике ↗</RouterLink></div></aside>
    </div>
  </div>
</template>
