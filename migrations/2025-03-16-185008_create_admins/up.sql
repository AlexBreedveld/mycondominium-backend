CREATE TABLE admins (
                           id UUID PRIMARY KEY NOT NULL,
                           first_name TEXT NOT NULL,
                           last_name TEXT NOT NULL,
                           phone TEXT,
                           email TEXT,
                           created_at TIMESTAMP NOT NULL,
                           updated_at TIMESTAMP NOT NULL
);