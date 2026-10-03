import { describe, it, expect } from 'vitest'
import { unlocked, nextLesson, nextInTrack } from './progression'
import type { Summary } from './types'
const lessons = [{id:'a',track:'intro',prerequisites:[]},{id:'b',track:'intro',prerequisites:['a']},{id:'x',track:'advanced',prerequisites:[]},{id:'y',track:'advanced',prerequisites:['x']}] as unknown as Summary[]
describe('v0.2 progression',()=>{
 it('tracks have independent entries',()=>{expect(unlocked(lessons,[],'a')).toBe(true);expect(unlocked(lessons,[],'x')).toBe(true);expect(unlocked(lessons,[],'y')).toBe(false)})
 it('enforces explicit prerequisites',()=>{expect(unlocked(lessons,['a'],'b')).toBe(true);expect(unlocked(lessons,['a'],'y')).toBe(false)})
 it('already completed lessons stay open',()=>{expect(unlocked(lessons,['b'],'b')).toBe(true)})
 it('next suggestion respects selected track',()=>{expect(nextLesson(lessons,[],'advanced')?.id).toBe('x');expect(nextLesson(lessons,['a','b'],'intro')).toBeUndefined()})
 it('navigation never crosses tracks by accident',()=>{expect(nextInTrack(lessons,'b')).toBeUndefined();expect(nextInTrack(lessons,'x')?.id).toBe('y')})
 it('unknown ids remain locked',()=>{expect(unlocked(lessons,[],'missing')).toBe(false)})
})
