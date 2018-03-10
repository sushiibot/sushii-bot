-- Your SQL goes here
ALTER TABLE users 
    DROP COLUMN profile_background_url,
    DROP COLUMN profile_bio,
    DROP COLUMN profile_bg_darken,
    DROP COLUMN profile_content_color,
    DROP COLUMN profile_content_opacity,
    DROP COLUMN profile_text_color,
    DROP COLUMN profile_accent_color;
