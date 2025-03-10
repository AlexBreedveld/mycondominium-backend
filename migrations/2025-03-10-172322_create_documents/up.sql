CREATE TABLE documents (
    id UUID PRIMARY KEY,
    title VARCHAR(150) NOT NULL,
    description TEXT,
    file_url TEXT NOT NULL,
    document_type VARCHAR(50),
    shared_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
