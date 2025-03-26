CREATE TABLE communities (
                            id        uuid not null UNIQUE
                                constraint communities_pk
                                    primary key,
                            name    varchar(50) NOT NULL,
                            short_name  varchar(25),
                            address text NOT NULL,
                            created_at TIMESTAMP NOT NULL,
                            updated_at TIMESTAMP NOT NULL
);