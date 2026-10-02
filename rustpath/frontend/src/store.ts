import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { api } from './api'
import { unlocked, nextLesson } from './progression'
import type { Course, Progress } from './types'
export const useLearning = defineStore('learning', () => {
  const course = ref<Course | null>(null)
  const progress = ref<Progress>({ completed: [], xp: 0, streak: 0, total_lessons: 0 })
  const loading = ref(true)
  const error = ref('')
  const next = computed(() => nextLesson(course.value?.lessons ?? [], progress.value.completed))
  const percent = computed(() => progress.value.total_lessons ? Math.round(progress.value.completed.length / progress.value.total_lessons * 100) : 0)
  function isUnlocked(id: string) { return unlocked(course.value?.lessons ?? [], progress.value.completed, id) }
  async function init() {
    loading.value = true; error.value = ''
    try {
      await api('/session', { method: 'POST' })
      const [c,p] = await Promise.all([api<Course>('/course'), api<Progress>('/progress')])
      course.value = c; progress.value = p
    } catch (e) { error.value = (e as Error).message }
    finally { loading.value = false }
  }
  return { course, progress, loading, error, next, percent, isUnlocked, init }
})
