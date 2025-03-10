CREATE TABLE votes (
    id UUID PRIMARY KEY,
    election_id INTEGER NOT NULL REFERENCES elections(id) ON DELETE CASCADE,
    resident_id INTEGER NOT NULL REFERENCES residents(id) ON DELETE CASCADE,
    vote_option VARCHAR(50) NOT NULL,
    voted_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (election_id, resident_id)
);
