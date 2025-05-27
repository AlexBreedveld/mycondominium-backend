CREATE TABLE password_reset (
    id uuid not null UNIQUE
        constraint password_reset_pk
            primary key,
    email text not null,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token text not null,
    created_at TIMESTAMP NOT NULL DEFAULT now()
)