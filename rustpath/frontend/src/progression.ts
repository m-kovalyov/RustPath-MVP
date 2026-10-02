import type { Summary } from './types'
export function unlocked(lessons: Summary[], completed: string[], id: string): boolean {
  const index = lessons.findIndex(l => l.id === id)
  return index >= 0 && (index === 0 || completed.includes(lessons[index - 1].id))
}
export function nextLesson(lessons: Summary[], completed: string[]): Summary | undefined {
  return lessons.find(l => !completed.includes(l.id))
}
