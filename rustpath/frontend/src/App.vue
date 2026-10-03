<script setup lang="ts">
import Icon from './components/Icon.vue'
import { onMounted } from 'vue'
import { useLearning } from './store'
const store = useLearning()
onMounted(store.init)
</script>
<template>
  <a class="skip" href="#main">К содержимому</a>
  <header class="topbar">
    <RouterLink to="/" class="brand" aria-label="RustPath — главная"><span class="brand-mark">r<span>_</span></span><span>rustpath<span class="brand-dot">.</span></span></RouterLink>
    <nav aria-label="Главное меню"><RouterLink to="/" exact-active-class="nav-active">Курс</RouterLink><RouterLink to="/review" active-class="nav-active">Повторение</RouterLink><RouterLink to="/lab" active-class="nav-active">Лаборатория</RouterLink><RouterLink to="/roadmap" active-class="nav-active">Маршрут</RouterLink></nav>
    <div class="header-progress"><Icon name="spark" class="xp-symbol" /><strong>{{ store.progress.xp }}</strong> XP</div>
  </header>
  <main id="main" tabindex="-1">
    <div v-if="store.loading" class="status-page" role="status"><span class="eyebrow">ГОТОВИМ ВАШ МАРШРУТ</span><h1>Маленькие шаги.<br>Настоящие навыки.</h1><p>Загружаем уроки и прогресс…</p></div>
    <div v-else-if="store.error" class="status-page" role="alert"><span class="eyebrow">СОЕДИНЕНИЕ ПРЕРВАНО</span><h1>Вернёмся к обучению?</h1><p>{{ store.error }}</p><button class="primary" @click="store.init">Попробовать снова</button></div>
    <RouterView v-else />
  </main>
  <footer><span>RustPath / учиться, создавать, понимать.</span><span>Гостевой прогресс связан с cookie этого браузера.<br>Не очищайте cookie: восстановление доступа пока не реализовано.</span></footer>
</template>
