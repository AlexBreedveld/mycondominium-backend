CREATE TABLE users (
                       id        uuid not null UNIQUE
                           constraint users_pk
                               primary key,
                       entity_id uuid not null,
                       entity_type VARCHAR(10) NOT NULL CHECK (entity_type IN ('admin', 'resident')),
                       admin_id uuid,
                       resident_id uuid,
                       password  text not null,
                       created_at TIMESTAMP NOT NULL,
                       updated_at TIMESTAMP NOT NULL,
                       CONSTRAINT users_entity_fk_admins FOREIGN KEY (admin_id)
                           REFERENCES public.admins(id) ON DELETE CASCADE,
                       CONSTRAINT users_entity_fk_residents FOREIGN KEY (resident_id)
                           REFERENCES public.residents(id) ON DELETE CASCADE,
                       CONSTRAINT users_entity_one_type_only CHECK (
                           (entity_type = 'admin' AND admin_id IS NOT NULL AND resident_id IS NULL)
                               OR
                           (entity_type = 'resident' AND resident_id IS NOT NULL AND admin_id IS NULL)
                           ),
                       CONSTRAINT users_entity_id_matches CHECK (
                           (entity_type = 'admin' AND entity_id = admin_id)
                               OR
                           (entity_type = 'resident' AND entity_id = resident_id)
                           )
);