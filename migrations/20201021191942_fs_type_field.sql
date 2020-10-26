-- Add migration script here
CREATE TYPE fs_type AS ENUM('zfs', 'ldiskfs');

ALTER TABLE IF EXISTS target add column fs_type fs_type;