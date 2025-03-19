CREATE TABLE announcements (
    id UUID PRIMARY KEY,
    title VARCHAR(150) NOT NULL,
    community_id uuid
        constraint announcements_communities_id_fk
            references public.communities,
    message TEXT NOT NULL,
    sent_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
