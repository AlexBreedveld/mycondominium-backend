CREATE TABLE maintenance_schedules (
    id UUID PRIMARY KEY,
    description TEXT NOT NULL,
    scheduled_date TIMESTAMP NOT NULL,
    status VARCHAR(20) NOT NULL,
    details TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
