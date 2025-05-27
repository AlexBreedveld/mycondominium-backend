CREATE TABLE maintenance_schedules
(
    id             UUID PRIMARY KEY,
    community_id   uuid
        constraint maintenance_schedules_communities_id_fk
            references public.communities ON DELETE CASCADE,
    description    TEXT        NOT NULL,
    scheduled_date TIMESTAMP   NOT NULL,
    status         VARCHAR(20) NOT NULL,
    details        TEXT,
    created_at     TIMESTAMP   NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at     TIMESTAMP   NOT NULL DEFAULT CURRENT_TIMESTAMP
);
