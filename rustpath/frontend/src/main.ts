import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createRouter, createWebHistory } from 'vue-router'
import App from './App.vue'
import './style.css'
const router = createRouter({ history: createWebHistory(), routes: [
  { path:'/', component:()=>import('./components/Home.vue') }, { path:'/learn/:id', component:()=>import('./components/Lesson.vue') }, { path:'/roadmap', component:()=>import('./components/Roadmap.vue') },
  { path:'/review', component:()=>import('./components/Review.vue') }, { path:'/lab', component:()=>import('./components/Lab.vue') },
  { path:'/:pathMatch(.*)*', redirect:'/' }
], scrollBehavior: () => ({top:0}) })
createApp(App).use(createPinia()).use(router).mount('#app')
