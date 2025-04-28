CREATE TABLE resident_invites (
    id UUID PRIMARY KEY NOT NULL,
    email TEXT NOT NULL UNIQUE,
    community_id uuid NOT NULL
       constraint user_roles_communities_id_fk
            references public.communities,
    key TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL
);