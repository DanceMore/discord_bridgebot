use crate::schema::channel_pairs;
use diesel::prelude::*;
use diesel::Queryable;

#[diesel(table_name = channel_pairs)]
#[derive(Queryable, Selectable, Insertable, Debug)]
pub struct ChannelPair {
    pub id: Option<i32>,
    pub channel1: i64,
    pub channel2: i64,
}

impl ChannelPair {
    pub fn new(channel1: i64, channel2: i64) -> Self {
        ChannelPair {
            id: None,
            channel1,
            channel2,
        }
    }
}
