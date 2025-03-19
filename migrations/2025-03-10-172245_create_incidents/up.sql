CREATE TABLE incidents (
    id UUID PRIMARY KEY,
    resident_id UUID REFERENCES residents(id),
    community_id uuid
        constraint incidents_communities_id_fk
            references public.communities,
    description TEXT NOT NULL,
    status VARCHAR(20) NOT NULL,
    report_date TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    resolution_date TIMESTAMP,
    notes TEXT
);
