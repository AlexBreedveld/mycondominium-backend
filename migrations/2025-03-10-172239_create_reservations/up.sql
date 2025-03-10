CREATE TABLE reservations (
    id UUID PRIMARY KEY,
    resident_id UUID NOT NULL REFERENCES residents(id) ON DELETE CASCADE,
    common_area_id UUID NOT NULL REFERENCES common_areas(id) ON DELETE CASCADE,
    reservation_date TIMESTAMP NOT NULL,
    start_time TIME NOT NULL,
    end_time TIME NOT NULL,
    status VARCHAR(20) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
