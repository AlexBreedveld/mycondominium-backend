CREATE TABLE users (
                       id        uuid not null UNIQUE
                           constraint users_pk
                               primary key,
                       entity_id uuid not null
                           constraint users_admin_id_fk
                               references public.admins
                           constraint users_residents_id_fk
                               references public.residents,
                       roles     text,
                       password  text not null,
                           created_at TIMESTAMP NOT NULL,
                           updated_at TIMESTAMP NOT NULL
);