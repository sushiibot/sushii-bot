-- Your SQL goes here
ALTER TABLE users 
    ADD COLUMN profile_background_url TEXT,
    ADD COLUMN profile_bio TEXT,
    ADD COLUMN profile_bg_darken TEXT,
    ADD COLUMN profile_content_color TEXT,
    ADD COLUMN profile_content_opacity TEXT,
    ADD COLUMN profile_text_color TEXT,
    ADD COLUMN profile_accent_color TEXT;
