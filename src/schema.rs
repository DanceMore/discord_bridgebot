// @generated automatically by Diesel CLI.

diesel::table! {
    channel_pairs (id) {
        id -> Int4,
        channel1 -> Int8,
        channel2 -> Int8,
    }
}
