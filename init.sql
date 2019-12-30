-- general conventions:
--
-- - inserted: when the record was inserted

-- IAM
CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  timezone VARCHAR NULL,
  inserted TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE identities (
  id SERIAL PRIMARY KEY,
  provider VARCHAR NOT NULL, -- eg. 'github'
  pid VARCHAR NOT NULL, -- user's id in that provider
  user_id INT NOT NULL REFERENCES users(id),
  email VARCHAR NOT NULL,
  username VARCHAR NOT NULL,
  inserted TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(provider, id)
);

-- CHANNELS

CREATE TABLE channels (
  id SERIAL PRIMARY KEY,
  channel_type VARCHAR NOT NULL, -- eg. github_release, etc..
  name VARCHAR NULL, -- name of the channel, eg. 'kubernetes/kubernetes'. format depends on type
  last_fetched TIMESTAMP WITH TIME ZONE NULL, -- last successful fetch
  inserted TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(channel_type, name) -- cannot have channel twice
);

CREATE TABLE updates (
  id BIGSERIAL PRIMARY KEY,
  channel_id INT REFERENCES channels(id),
  title VARCHAR NOT NULL,
  url VARCHAR NULL, -- direct link to update
  published TIMESTAMP WITH TIME ZONE NULL, -- when the update was published
  inserted TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(channel_id, title, published) -- title could be duplicate, but not for the same published date
);

CREATE TABLE subscriptions (
  id SERIAL PRIMARY KEY,
  email VARCHAR NOT NULL,
  channel_id INT REFERENCES channels(id),
  user_id INT REFERENCES users(id),
  frequency VARCHAR NOT NULL, -- daily or weekly
  day VARCHAR NULL, -- any three-letter day: set if frequency is weekly, we also have a day
  time TIME WITHOUT TIME ZONE NOT NULL, -- timezone is based on user profile
  inserted TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(channel_id, user_id) -- user can subscribe to channel only once
);

-- the idea is that we look at all subscriptions and create
-- digests with the next due date (eg. subscription A is daily
-- at 9am, so we add a digest with due = 'today 9am' and sent = NULL.)
-- after that, we look at all the digests and send those where due is
-- before now and sent is NULL.
CREATE TABLE digests (
  id BIGSERIAL PRIMARY KEY,
  subscription_id INT REFERENCES subscriptions(id),
  due TIMESTAMP WITH TIME ZONE NOT NULL, -- when the digest shoud be sent
  sent TIMESTAMP WITH TIME ZONE, -- null if not sent yet, otherwise set to the send time
  inserted TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- per subscription, we can only have one unsent digest
CREATE UNIQUE INDEX digests_only_one_unsent_idx ON digests (subscription_id) WHERE sent IS NULL;
