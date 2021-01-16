table! {
    past_dailies (id) {
        id -> Int4,
        posted_on -> Date,
        robot_id -> Int4,
    }
}

table! {
    reply_tweets (id) {
        id -> Int4,
        request_tweet_id -> Int8,
        request_tweet_time -> Timestamp,
        reply_tweet_id -> Int8,
        reply_tweet_time -> Timestamp,
        user_id -> Int8,
        user_handle -> Text,
        robot_id -> Int4,
    }
}

table! {
    robot_groups (id) {
        id -> Int4,
        tweet_id -> Int8,
        tweet_time -> Timestamp,
        image_url -> Nullable<Text>,
        original_names -> Text,
        body -> Text,
        original_body -> Text,
        alt -> Nullable<Text>,
    }
}

table! {
    robots (id) {
        id -> Int4,
        robot_group_id -> Int4,
        robot_number -> Int4,
        prefix -> Text,
        suffix -> Text,
        suffix_singular -> Text,
        name_raw -> Text,
    }
}

table! {
    scheduled_dailies (id) {
        id -> Int4,
        post_on -> Date,
        robot_id -> Int4,
    }
}

table! {
    tags (id) {
        id -> Int4,
        robot_group_id -> Int4,
        tag -> Text,
    }
}

joinable!(past_dailies -> robots (robot_id));
joinable!(reply_tweets -> robots (robot_id));
joinable!(robots -> robot_groups (robot_group_id));
joinable!(scheduled_dailies -> robots (robot_id));
joinable!(tags -> robot_groups (robot_group_id));

allow_tables_to_appear_in_same_query!(
    past_dailies,
    reply_tweets,
    robot_groups,
    robots,
    scheduled_dailies,
    tags,
);
