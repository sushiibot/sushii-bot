-- Firstly, remove PRIMARY KEY attribute of former PRIMARY KEY
ALTER TABLE levels DROP CONSTRAINT levels_pkey;
-- Then remove the current primary key.
ALTER TABLE levels DROP COLUMN id;
-- Lastly set your new PRIMARY KEY
ALTER TABLE levels ADD PRIMARY KEY (user_id, guild_id);
