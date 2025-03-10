CREATE TABLE incidents (
    id UUID PRIMARY KEY,
    resident_id UUID REFERENCES residents(id),
    description TEXT NOT NULL,
    status VARCHAR(20) NOT NULL,
    report_date TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    resolution_date TIMESTAMP,
    notes TEXT
);
