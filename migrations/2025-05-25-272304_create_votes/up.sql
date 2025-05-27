CREATE TABLE votes
(
    id          UUID PRIMARY KEY NOT NULL,
    election_id UUID             NOT NULL REFERENCES elections (id) ON DELETE CASCADE,
    user_id     UUID             NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    vote_option UUID             NOT NULL REFERENCES election_candidates (id) ON DELETE CASCADE,
    voted_at    TIMESTAMP        NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (election_id, user_id)
);
