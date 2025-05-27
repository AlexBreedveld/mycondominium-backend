CREATE TABLE election_candidates
(
    id          UUID PRIMARY KEY NOT NULL,
    election_id UUID             NOT NULL REFERENCES elections (id) ON DELETE CASCADE,
    name        VARCHAR(150)     NOT NULL
);
