use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize)]
pub struct Course {
    pub version: String,
    pub tracks: Vec<Track>,
    pub modules: Vec<Module>,
    pub lessons: Vec<Lesson>,
    pub roadmap: Vec<RoadmapModule>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub title: String,
    pub description: String,
    pub level: String,
    pub entry: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Module {
    pub id: String,
    pub track: String,
    pub title: String,
    pub description: String,
    pub number: usize,
    pub outcome: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct RoadmapModule {
    pub title: String,
    pub description: String,
    pub outcome: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Reading {
    pub title: String,
    pub url: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Theory {
    pub heading: String,
    pub text: String,
    pub example: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Quiz {
    pub question: String,
    pub options: Vec<String>,
    #[serde(skip_serializing)]
    pub correct: usize,
    #[serde(skip_serializing)]
    pub explanation: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Lesson {
    pub id: String,
    pub module: String,
    pub track: String,
    pub kind: String,
    pub prerequisites: Vec<String>,
    pub title: String,
    pub minutes: u32,
    pub xp: i32,
    pub summary: String,
    pub outcome: String,
    pub common_mistake: String,
    pub theory: Vec<Theory>,
    pub task: String,
    pub starter: String,
    pub hints: Vec<String>,
    pub test_labels: Vec<String>,
    pub readings: Vec<Reading>,
    pub visual: Option<String>,
    pub quiz: Option<Quiz>,
    #[serde(skip_serializing)]
    pub tests: String,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
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
        self.lesson(id).is_some_and(|l| {
            completed.contains(&l.id) || l.prerequisites.iter().all(|p| completed.contains(p))
        })
    }
}
#[derive(Clone, Serialize)]
pub struct LessonSummary {
    pub id: String,
    pub module: String,
    pub track: String,
    pub kind: String,
    pub prerequisites: Vec<String>,
    pub title: String,
    pub minutes: u32,
    pub xp: i32,
    pub summary: String,
    pub outcome: String,
}
impl From<&Lesson> for LessonSummary {
    fn from(l: &Lesson) -> Self {
        Self {
            id: l.id.clone(),
            module: l.module.clone(),
            track: l.track.clone(),
            kind: l.kind.clone(),
            prerequisites: l.prerequisites.clone(),
            title: l.title.clone(),
            minutes: l.minutes,
            xp: l.xp,
            summary: l.summary.clone(),
            outcome: l.outcome.clone(),
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
        assert!(c.unlocked("intro-rust", &[]));
        assert!(!c.unlocked("intro-main", &[]));
        assert!(c.unlocked("intro-main", &["intro-rust".into()]));
        assert!(!c.unlocked("missing", &[]));
    }
    #[test]
    fn tracks_are_independent_and_legacy_progress_survives() {
        let c = Course::load();
        assert!(c.unlocked("hello", &[]));
        assert!(c.unlocked("adv-min", &[]));
        assert!(c.unlocked("arithmetic", &["hello".into()]));
        assert!(c.unlocked("composition", &["composition".into()]));
        let old: serde_json::Value =
            serde_json::from_str(include_str!("../../tests/legacy-course.json")).unwrap();
        for l in old["lessons"].as_array().unwrap() {
            let id = l["id"].as_str().unwrap();
            let new = c.lesson(id).unwrap();
            assert_eq!(new.xp, l["xp"].as_i64().unwrap() as i32);
            assert_eq!(new.tests, l["tests"].as_str().unwrap());
        }
    }
    #[test]
    fn answers_never_leak() {
        let c = Course::load();
        for l in &c.lessons {
            let v = serde_json::to_value(l).unwrap();
            assert!(v.get("tests").is_none());
            assert!(v.get("solution").is_none());
            if l.kind == "quiz" {
                assert!(v["quiz"].get("correct").is_none());
                assert!(v["quiz"].get("explanation").is_none());
            }
        }
    }
    #[test]
    fn course_has_unique_ids_and_complete_content() {
        let c = Course::load();
        let ids: std::collections::HashSet<_> = c.lessons.iter().map(|l| &l.id).collect();
        assert_eq!(ids.len(), 44);
        for l in &c.lessons {
            assert!(
                c.modules
                    .iter()
                    .any(|m| m.id == l.module && m.track == l.track)
            );
            assert!(!l.theory.is_empty() && !l.hints.is_empty() && !l.readings.is_empty());
            for p in &l.prerequisites {
                assert_ne!(p, &l.id);
                assert_eq!(c.lesson(p).unwrap().track, l.track);
            }
            if l.kind == "code" {
                assert!(!l.tests.is_empty() && !l.solution.is_empty());
            } else {
                let q = l.quiz.as_ref().unwrap();
                assert!(q.correct < q.options.len());
                assert!(!q.explanation.is_empty());
            }
        }
    }
}
