import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { api } from './api'
import { unlocked, nextLesson } from './progression'
import type { Course, Progress } from './types'
const preference=()=>{try{return localStorage.getItem('rustpath.track')}catch{return null}}
export const useLearning = defineStore('learning',()=>{
 const course=ref<Course|null>(null), progress=ref<Progress>({completed:[],xp:0,streak:0,total_lessons:0})
 const bookmarks=ref<string[]>([]),selectedTrack=ref(preference() ?? 'intro'),loading=ref(true),error=ref(''),bookmarkBusy=ref(false)
 const trackLessons=computed(()=>course.value?.lessons.filter(l=>l.track===selectedTrack.value) ?? [])
 const completedTrack=computed(()=>trackLessons.value.filter(l=>progress.value.completed.includes(l.id)).length)
 const next=computed(()=>nextLesson(course.value?.lessons ?? [],progress.value.completed,selectedTrack.value))
 const percent=computed(()=>trackLessons.value.length ? Math.round(completedTrack.value/trackLessons.value.length*100):0)
 const currentTrack=computed(()=>course.value?.tracks.find(t=>t.id===selectedTrack.value))
 function isUnlocked(id:string){return unlocked(course.value?.lessons ?? [],progress.value.completed,id)}
 function selectTrack(id:string){if(!course.value?.tracks.some(t=>t.id===id))return;selectedTrack.value=id;try{localStorage.setItem('rustpath.track',id)}catch{}}
 async function toggleBookmark(id:string){
  if(bookmarkBusy.value)return
  bookmarkBusy.value=true
  try{const marked=bookmarks.value.includes(id);await api(`/lessons/${id}/bookmark`,{method:marked?'DELETE':'PUT'});bookmarks.value=marked?bookmarks.value.filter(v=>v!==id):[...bookmarks.value,id]}
  finally{bookmarkBusy.value=false}
 }
 async function init(){loading.value=true;error.value='';try{
  await api('/session',{method:'POST'})
  const [c,p,b]=await Promise.all([api<Course>('/course'),api<Progress>('/progress'),api<string[]>('/bookmarks')])
  course.value=c;progress.value=p;bookmarks.value=b
  if(!preference() && p.completed.some(id=>c.lessons.some(l=>l.id===id && l.track==='core')))selectedTrack.value='core'
  if(!c.tracks.some(t=>t.id===selectedTrack.value))selectedTrack.value='intro'
 }catch(e){error.value=(e as Error).message}finally{loading.value=false}}
 return{course,progress,bookmarks,selectedTrack,loading,error,bookmarkBusy,trackLessons,completedTrack,next,percent,currentTrack,isUnlocked,selectTrack,toggleBookmark,init}
})
