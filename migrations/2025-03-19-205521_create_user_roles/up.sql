CREATE TABLE user_roles (
                       id        uuid not null UNIQUE
                           constraint user_roles_pk
                               primary key,
                       user_id uuid not null
                           constraint user_roles_users_id_fk
                               references public.users ON DELETE CASCADE,
                       role     text NOT NULL,
                       community_id uuid
                           constraint user_roles_communities_id_fk
                               references public.communities,
                       created_at TIMESTAMP NOT NULL,
                       updated_at TIMESTAMP NOT NULL
);