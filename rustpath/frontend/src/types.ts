export interface Track { id: string; title: string; description: string; level: string; entry: string }
export interface Module { id: string; track: string; title: string; description: string; number: number; outcome: string }
export interface Summary { id: string; module: string; track: string; kind: 'code'|'quiz'; prerequisites: string[]; title: string; minutes: number; xp: number; summary: string; outcome: string }
export interface Roadmap { title: string; description: string; outcome: string }
export interface Course { version: string; tracks: Track[]; modules: Module[]; lessons: Summary[]; roadmap: Roadmap[] }
export interface Theory { heading: string; text: string; example: string | null }
export interface Reading { title: string; url: string }
export interface Quiz { question: string; options: string[] }
export interface Lesson extends Summary { theory: Theory[]; task: string; starter: string; hints: string[]; test_labels: string[]; common_mistake: string; readings: Reading[]; visual: string|null; quiz: Quiz|null }
export interface Progress { completed: string[]; xp: number; streak: number; total_lessons: number }
export interface Evaluation { passed: boolean; output: string; duration_ms: number }
export interface Submission { evaluation: Evaluation; awarded_xp: number; progress: Progress }
