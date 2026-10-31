CREATE SCHEMA cw_users;

CREATE TABLE cw_users.users (
    id uuid PRIMARY KEY,
    email text NOT NULL,
    CONSTRAINT users_email_uq UNIQUE (email)
);

ALTER TABLE cw_users.users ADD COLUMN active boolean NOT NULL DEFAULT true;
