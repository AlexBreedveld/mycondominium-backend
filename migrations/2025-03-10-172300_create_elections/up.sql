CREATE TABLE elections
(
    id           UUID PRIMARY KEY NOT NULL,
    community_id uuid             NOT NULL
        constraint elections_communities_id_fk
            references public.communities ON DELETE CASCADE,
    title        VARCHAR(150)     NOT NULL,
    description  TEXT,
    start_date   TIMESTAMP        NOT NULL,
    end_date     TIMESTAMP        NOT NULL,
    created_at   TIMESTAMP        NOT NULL DEFAULT CURRENT_TIMESTAMP
);
