CREATE TABLE common_areas (
    id UUID PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    community_id uuid
        constraint common_areas_communities_id_fk
            references public.communities,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
