CREATE TABLE blogs (
  id SERIAL PRIMARY KEY,
  url VARCHAR NOT NULL UNIQUE, -- url incl scheme
  title VARCHAR NULL, -- name of the blog, set on first successful fetch
  author VARCHAR NULL, -- author of the blog, set on first successful fetch
  last_fetched TIMESTAMP WITH TIME ZONE NULL -- last successful fetch
);

CREATE TABLE posts (
  id BIGSERIAL PRIMARY KEY,
  blog_id INT REFERENCES blogs(id),
  title VARCHAR NOT NULL,
  author VARCHAR NULL, -- could be different from blog for guest posts
  created TIMESTAMP WITH TIME ZONE NULL, -- when the post was created
  url VARCHAR NULL -- direct link to post
);
