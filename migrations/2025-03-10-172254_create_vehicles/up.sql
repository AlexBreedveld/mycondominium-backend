CREATE TABLE vehicles (
    id UUID PRIMARY KEY,
    resident_id UUID NOT NULL REFERENCES residents(id) ON DELETE CASCADE,
    license_plate VARCHAR(20) NOT NULL UNIQUE,
    model VARCHAR(100),
    color VARCHAR(50),
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);
