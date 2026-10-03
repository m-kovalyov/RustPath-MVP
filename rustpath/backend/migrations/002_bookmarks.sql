CREATE TABLE bookmarks (
    learner_id UUID NOT NULL REFERENCES learners(id) ON DELETE CASCADE,
    lesson_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (learner_id, lesson_id)
);
