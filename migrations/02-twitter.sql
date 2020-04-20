ALTER TABLE channels RENAME COLUMN url TO ext_id;
ALTER TABLE channels
  ADD COLUMN verified BOOL NOT NULL DEFAULT FALSE,
  ADD COLUMN last_cleaned TIMESTAMP WITH TIME ZONE NULL;

UPDATE channels SET ext_id = name WHERE channel_type = 'github_release';

ALTER TABLE updates ADD COLUMN ext_id VARCHAR NULL;

ALTER TABLE channels DROP CONSTRAINT channels_channel_type_name_key;
ALTER TABLE channels ADD CONSTRAINT channels_channel_type_ext_id UNIQUE(channel_type, ext_id);
