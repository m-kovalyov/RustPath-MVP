use crate::{domain::Progress, error::AppError};
use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait LearningRepository: Send + Sync {
    async fn progress(&self, learner: Uuid, total: usize) -> Result<Progress, AppError>;
    async fn complete(
        &self,
        learner: Uuid,
        lesson: &str,
        xp: i32,
        passed: bool,
    ) -> Result<i32, AppError>;
}
pub struct PgLearningRepository(pub PgPool);
#[async_trait]
impl LearningRepository for PgLearningRepository {
    async fn progress(&self, learner: Uuid, total: usize) -> Result<Progress, AppError> {
        let rows: Vec<(String, i32)> = sqlx::query_as(
            "SELECT lesson_id, xp FROM completions WHERE learner_id = $1 ORDER BY completed_at",
        )
        .bind(learner)
        .fetch_all(&self.0)
        .await?;
        // Streak includes today's activity or yesterday's still-active streak, UTC calendar days.
        let dates: Vec<(NaiveDate,)> = sqlx::query_as("SELECT DISTINCT (created_at AT TIME ZONE 'UTC')::date AS d FROM submissions WHERE learner_id = $1 AND passed ORDER BY d DESC")
            .bind(learner).fetch_all(&self.0).await?;
        let xp = rows.iter().map(|(_, xp)| i64::from(*xp)).sum();
        Ok(Progress {
            completed: rows.into_iter().map(|(id, _)| id).collect(),
            xp,
            streak: streak(
                &dates.into_iter().map(|(d,)| d).collect::<Vec<_>>(),
                Utc::now().date_naive(),
            ),
            total_lessons: total,
        })
    }
    async fn complete(
        &self,
        learner: Uuid,
        lesson: &str,
        xp: i32,
        passed: bool,
    ) -> Result<i32, AppError> {
        let mut tx = self.0.begin().await?;
        sqlx::query(
            "INSERT INTO submissions(id, learner_id, lesson_id, passed) VALUES($1,$2,$3,$4)",
        )
        .bind(Uuid::new_v4())
        .bind(learner)
        .bind(lesson)
        .bind(passed)
        .execute(&mut *tx)
        .await?;
        let awarded = if passed {
            sqlx::query("INSERT INTO completions(learner_id,lesson_id,xp) VALUES($1,$2,$3) ON CONFLICT DO NOTHING")
                .bind(learner).bind(lesson).bind(xp).execute(&mut *tx).await?.rows_affected() > 0
        } else {
            false
        };
        tx.commit().await?;
        Ok(if awarded { xp } else { 0 })
    }
}
fn streak(dates: &[NaiveDate], today: NaiveDate) -> i64 {
    let Some(&latest) = dates.first() else {
        return 0;
    };
    if latest != today && Some(latest) != today.pred_opt() {
        return 0;
    }
    let mut expected = latest;
    let mut result = 0;
    for &d in dates {
        if d != expected {
            break;
        }
        result += 1;
        let Some(previous) = expected.pred_opt() else {
            break;
        };
        expected = previous;
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn streak_respects_gaps_and_grace_day() {
        let d = NaiveDate::from_ymd_opt(2026, 10, 2).unwrap();
        assert_eq!(streak(&[], d), 0);
        assert_eq!(streak(&[d, d.pred_opt().unwrap()], d), 2);
        assert_eq!(streak(&[d.pred_opt().unwrap()], d), 1);
        assert_eq!(streak(&[d - chrono::Duration::days(2)], d), 0);
    }
}
