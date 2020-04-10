CREATE TABLE lists (
  id SERIAL PRIMARY KEY,
  name VARCHAR NOT NULL,
  creator INT REFERENCES users(id),
  inserted TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE lists_channels (
  list_id INT REFERENCES lists(id),
  channel_id INT REFERENCES channels(id),
  inserted TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY(list_id, channel_id)
);

ALTER TABLE subscriptions 
  ADD COLUMN timezone VARCHAR NULL, 
  ADD COLUMN list_id INT REFERENCES lists(id),
  ADD CONSTRAINT subscriptions_list_id_user_id_key UNIQUE (list_id, user_id);