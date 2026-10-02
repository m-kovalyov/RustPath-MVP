use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize)]
pub struct Course {
    pub modules: Vec<Module>,
    pub lessons: Vec<Lesson>,
    pub roadmap: Vec<RoadmapModule>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Module {
    pub id: String,
    pub title: String,
    pub description: String,
    pub number: usize,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct RoadmapModule {
    pub title: String,
    pub description: String,
    pub outcome: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Theory {
    pub heading: String,
    pub text: String,
    pub example: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Lesson {
    pub id: String,
    pub module: String,
    pub title: String,
    pub minutes: u32,
    pub xp: i32,
    pub summary: String,
    pub theory: Vec<Theory>,
    pub task: String,
    pub starter: String,
    pub hints: Vec<String>,
    pub test_labels: Vec<String>,
    // Never serialized into public responses.
    #[serde(skip_serializing)]
    pub tests: String,
    #[serde(skip_serializing)]
    #[allow(dead_code)] // Reference implementation is only used by curriculum QA.
    pub solution: String,
}
impl Course {
    pub fn load() -> Self {
        serde_json::from_str(include_str!("../course.json")).expect("valid bundled course")
    }
    pub fn lesson(&self, id: &str) -> Option<&Lesson> {
        self.lessons.iter().find(|l| l.id == id)
    }
    pub fn unlocked(&self, id: &str, completed: &[String]) -> bool {
        self.lessons
            .iter()
            .position(|l| l.id == id)
            .is_some_and(|i| i == 0 || completed.contains(&self.lessons[i - 1].id))
    }
}
#[derive(Clone, Serialize)]
pub struct LessonSummary {
    pub id: String,
    pub module: String,
    pub title: String,
    pub minutes: u32,
    pub xp: i32,
    pub summary: String,
}
impl From<&Lesson> for LessonSummary {
    fn from(l: &Lesson) -> Self {
        Self {
            id: l.id.clone(),
            module: l.module.clone(),
            title: l.title.clone(),
            minutes: l.minutes,
            xp: l.xp,
            summary: l.summary.clone(),
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Evaluation {
    pub passed: bool,
    pub output: String,
    pub duration_ms: u64,
}
#[derive(Serialize)]
pub struct Progress {
    pub completed: Vec<String>,
    pub xp: i64,
    pub streak: i64,
    pub total_lessons: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prerequisites_are_enforced() {
        let c = Course::load();
        assert!(c.unlocked(&c.lessons[0].id, &[]));
        assert!(!c.unlocked(&c.lessons[1].id, &[]));
        assert!(c.unlocked(&c.lessons[1].id, &[c.lessons[0].id.clone()]));
        assert!(!c.unlocked("missing", &[]));
    }
    #[test]
    fn answers_never_leak() {
        let c = Course::load();
        let value = serde_json::to_value(&c.lessons[0]).unwrap();
        assert!(value.get("tests").is_none());
        assert!(value.get("solution").is_none());
    }
    #[test]
    fn course_has_unique_ids_and_complete_content() {
        let c = Course::load();
        let ids: std::collections::HashSet<_> = c.lessons.iter().map(|l| &l.id).collect();
        assert_eq!(ids.len(), c.lessons.len());
        for l in &c.lessons {
            assert!(c.modules.iter().any(|m| m.id == l.module));
            assert!(!l.tests.is_empty() && !l.solution.is_empty());
            assert!(!l.theory.is_empty() && !l.hints.is_empty());
        }
    }
}
