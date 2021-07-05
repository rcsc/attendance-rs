-- Add migration script here
CREATE TYPE token_capability AS ENUM ('collector', 'viewer', 'administrator');
CREATE TABLE tokens(
    uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    description TEXT NOT NULL,
    initial_valid_time TIMESTAMP WITH TIME ZONE,
    expiration_time TIMESTAMP WITH TIME ZONE,
    create_time TIMESTAMP WITH TIME ZONE NOT NULL,
    capability token_capability NOT NULL
);
