CREATE TABLE learners (
    id UUID PRIMARY KEY,
    token_hash TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE submissions (
    id UUID PRIMARY KEY,
    learner_id UUID NOT NULL REFERENCES learners(id) ON DELETE CASCADE,
    lesson_id TEXT NOT NULL,
    passed BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX submissions_learner_time ON submissions(learner_id, created_at DESC);
CREATE TABLE completions (
    learner_id UUID NOT NULL REFERENCES learners(id) ON DELETE CASCADE,
    lesson_id TEXT NOT NULL,
    xp INTEGER NOT NULL CHECK (xp > 0),
    completed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY(learner_id, lesson_id)
);
CREATE TABLE drafts (
    learner_id UUID NOT NULL REFERENCES learners(id) ON DELETE CASCADE,
    lesson_id TEXT NOT NULL,
    code TEXT NOT NULL CHECK (octet_length(code) <= 16384),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY(learner_id, lesson_id)
);
