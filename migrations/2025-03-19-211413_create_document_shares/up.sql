CREATE TABLE document_shares (
                            id        uuid NOT NULL UNIQUE
                                constraint document_shares_pk
                                    primary key,
                            user_id uuid NOT NULL
                                constraint document_shares_users_id_fk
                                    references public.users,
                            document_id uuid NOT NULL
                                constraint document_shares_documents_id_fk
                                    references public.documents,
                            read_only     bool NOT NULL
);