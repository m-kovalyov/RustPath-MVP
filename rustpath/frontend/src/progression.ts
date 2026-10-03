import type { Summary } from './types'
export function unlocked(lessons: Summary[], completed: string[], id: string): boolean {
  const lesson = lessons.find(l => l.id === id)
  return !!lesson && (completed.includes(id) || lesson.prerequisites.every(p => completed.includes(p)))
}
export function nextLesson(lessons: Summary[], completed: string[], track?: string): Summary | undefined {
  return lessons.find(l => (!track || l.track===track) && !completed.includes(l.id) && unlocked(lessons,completed,l.id))
}
export function nextInTrack(lessons:Summary[], id:string):Summary|undefined {
  const lesson=lessons.find(l=>l.id===id)
  const trackLessons=lessons.filter(l=>l.track===lesson?.track)
  const index=trackLessons.findIndex(l=>l.id===id)
  return index>=0 ? trackLessons[index+1] : undefined
}
