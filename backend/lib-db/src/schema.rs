table! {
    users(id) {
        id -> Integer,
    }
}

table! {
    identities(id) {
        id -> Integer,
        provider -> Text,
        pid -> Text,
        user_id -> Integer,
        email -> Text,
        username -> Text,
    }
}

table! {
    channels (id) {
        id -> Integer,
        channel_type -> Text,
        name -> Text,
        last_fetched -> Nullable<Timestamptz>,
        inserted -> Timestamptz,
    }
}

table! {
    updates (id) {
        id -> BigInt,
        channel_id -> Integer,
        title -> Text,
        url -> Text,
        published -> Timestamptz,
        inserted -> Timestamptz,
    }
}

table! {
    subscriptions(id) {
      id -> Integer,
      email -> Text,
      channel_id -> Integer,
      user_id -> Integer,
      frequency -> Text,
      day -> Nullable<Text>,
      time -> Time,
    }
}

table! {
    digests(id) {
      id -> BigInt,
      subscription_id -> Integer,
      due -> Timestamptz,
      sent -> Nullable<Timestamptz>,
    }
}

allow_tables_to_appear_in_same_query!(subscriptions, digests);
allow_tables_to_appear_in_same_query!(subscriptions, channels);
allow_tables_to_appear_in_same_query!(users, identities);
