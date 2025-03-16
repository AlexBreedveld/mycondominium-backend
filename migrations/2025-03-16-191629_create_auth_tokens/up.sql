create table auth_tokens
(
    user_id     uuid      not null
        constraint auth_tokens_user_id_fk
            references public.users,
    id uuid      not null
        constraint auth_tokens_pk
            primary key,
    time_added timestamp not null,
    active     boolean   not null,
    time_last_used timestamp not null,
    device     text,
    browser     text,
    version     text,
    cpu_arch    text
);