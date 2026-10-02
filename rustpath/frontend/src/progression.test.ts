import { describe, it, expect } from 'vitest'
import { unlocked, nextLesson } from './progression'
import type { Summary } from './types'
const lessons = ['a','b','c'].map(id => ({ id, module:'m',title:id,minutes:5,xp:50,summary:'' })) as Summary[]
describe('learning progression', () => {
  it('only unlocks the next level', () => { expect(unlocked(lessons, [], 'a')).toBe(true); expect(unlocked(lessons, [], 'b')).toBe(false); expect(unlocked(lessons, ['a'], 'b')).toBe(true); expect(unlocked(lessons, ['a'], 'c')).toBe(false) })
  it('finds incomplete levels, not a hardcoded lesson', () => { expect(nextLesson(lessons,['a'])?.id).toBe('b'); expect(nextLesson(lessons,['a','b','c'])).toBeUndefined() })
  it('rejects unknown ids', () => { expect(unlocked(lessons, [], 'missing')).toBe(false) })
})
