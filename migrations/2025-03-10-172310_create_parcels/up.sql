CREATE TABLE parcels (
    id UUID PRIMARY KEY,
    resident_id INTEGER NOT NULL REFERENCES residents(id) ON DELETE CASCADE,
    parcel_type VARCHAR(50) NOT NULL,
    description TEXT,
    arrival_date TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    received BOOLEAN NOT NULL DEFAULT FALSE,
    received_at TIMESTAMP
);
