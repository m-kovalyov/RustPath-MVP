export interface Module { id: string; title: string; description: string; number: number }
export interface Summary { id: string; module: string; title: string; minutes: number; xp: number; summary: string }
export interface Roadmap { title: string; description: string; outcome: string }
export interface Course { modules: Module[]; lessons: Summary[]; roadmap: Roadmap[] }
export interface Theory { heading: string; text: string; example: string | null }
export interface Lesson extends Summary { theory: Theory[]; task: string; starter: string; hints: string[]; test_labels: string[] }
export interface Progress { completed: string[]; xp: number; streak: number; total_lessons: number }
export interface Evaluation { passed: boolean; output: string; duration_ms: number }
export interface Submission { evaluation: Evaluation; awarded_xp: number; progress: Progress }
