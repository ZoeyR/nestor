table! {
    factoids (id) {
        id -> Integer,
        label -> Text,
        intent -> Text,
        description -> Text,
        nickname -> Text,
        timestamp -> Timestamp,
        locked -> Bool,
    }
}
