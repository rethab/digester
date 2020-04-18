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
  ext_id VARCHAR NOT NULL, -- identifies a channel externally (eg. twitter handle, rss url)
  channel_type VARCHAR NOT NULL, -- eg. github_release, etc..
  name VARCHAR NULL, -- human readable name of the channel, eg. 'kubernetes/kubernetes'. format depends on type.
  link VARCHAR NOT NULL, -- link of the website eg. blog.acolyer.com or github.com/kubernetes/kubernetes
  verified BOOL NOT NULL, -- if true, this twitter account is verified
  last_fetched TIMESTAMP WITH TIME ZONE NULL, -- last successful fetch
  inserted TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(channel_type, ext_id) -- cannot have channel twice
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

-- LISTS

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

CREATE TABLE subscriptions (
  id SERIAL PRIMARY KEY,
  email VARCHAR NOT NULL,
  timezone VARCHAR NULL, -- timezone for anonymous users, other have it in the profile
  -- depending on whether the subscription is for a channel or a list,
  -- the respective attribute is set
  channel_id INT REFERENCES channels(id),
  list_id INT REFERENCES lists(id),
  user_id INT REFERENCES users(id),
  frequency VARCHAR NOT NULL, -- daily or weekly
  day VARCHAR NULL, -- any three-letter day: set if frequency is weekly, we also have a day
  time TIME WITHOUT TIME ZONE NOT NULL, -- timezone is based on user profile
  inserted TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(channel_id, user_id), -- user can subscribe to channel only once
  UNIQUE(list_id, user_id) -- user can subscribe to list only once
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

-- ANONYMOUS SUBSCRIPTIONS

-- anonymous subscriptions land here. as soon as the user clicks on the
-- confirmation link in the e-mail, the subscription is moved to the
-- subscriptions table
CREATE TABLE pending_subscriptions (
  id SERIAL PRIMARY KEY,
  email VARCHAR NOT NULL,
  timezone VARCHAR NOT NULL,
  -- only works for lists, not for individual channels
  list_id INT NOT NULL REFERENCES lists(id),
  token VARCHAR NULL,
  activation_email_sent TIMESTAMP WITH TIME ZONE NULL,
  -- same as regular subscriptions
  frequency VARCHAR NOT NULL,
  day VARCHAR NULL,
  time TIME WITHOUT TIME ZONE NOT NULL,
  inserted TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(list_id, email) -- user can subscribe to list only once
);
