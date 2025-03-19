CREATE TABLE invoices (
    id UUID PRIMARY KEY,
    resident_id UUID NOT NULL REFERENCES residents(id) ON DELETE CASCADE,
    community_id uuid
        constraint invoices_communities_id_fk
            references public.communities,
    issue_date DATE NOT NULL DEFAULT CURRENT_DATE,
    due_date DATE NOT NULL,
    amount NUMERIC(10, 2) NOT NULL,
    status VARCHAR(20) NOT NULL,
    paid_date DATE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
