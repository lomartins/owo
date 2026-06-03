-- Phase E: personalization for two-person households.
-- partner_name and photo_url are optional, free-form.
-- photo_url stores a relative path served by Axum at /uploads/...

ALTER TABLE users ADD COLUMN partner_name TEXT;
ALTER TABLE users ADD COLUMN photo_url TEXT;
