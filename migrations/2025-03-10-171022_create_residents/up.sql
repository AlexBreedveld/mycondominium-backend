CREATE TABLE residents (
    id UUID PRIMARY KEY NOT NULL,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    unit_number VARCHAR(20),
    address TEXT,
    phone TEXT,
    email TEXT NOT NULL,
    date_of_birth DATE,
    resident_since TIMESTAMP NOT NULL,
    is_active BOOLEAN NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);