CREATE TABLE parcels (
    id UUID PRIMARY KEY,
    resident_id UUID NOT NULL REFERENCES residents(id) ON DELETE CASCADE,
    community_id uuid
        constraint parcels_communities_id_fk
            references public.communities,
    parcel_type VARCHAR(50) NOT NULL,
    description TEXT,
    arrival_date TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    received BOOLEAN NOT NULL DEFAULT FALSE,
    received_at TIMESTAMP
);
