-- Firstly, remove PRIMARY KEY attribute of former PRIMARY KEY
ALTER TABLE levels DROP CONSTRAINT levels_pkey;
-- Then remove the current primary key.
ALTER TABLE levels ADD COLUMN id SERIAL;
-- Lastly set your new PRIMARY KEY
ALTER TABLE levels ADD PRIMARY KEY (id);
