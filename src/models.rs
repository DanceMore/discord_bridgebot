use crate::schema::channel_pairs;
use diesel::prelude::*;
use diesel::Queryable;
use diesel::Selectable;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = channel_pairs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChannelPair {
    pub id: i32,
    pub channel1: i64,
    pub channel2: i64,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = channel_pairs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableChannelPair {
    pub id: Option<i32>,
    pub channel1: i64,
    pub channel2: i64,
}

impl InsertableChannelPair {
    pub fn new(channel1: i64, channel2: i64) -> Self {
        InsertableChannelPair{
            id: None,
            channel1,
            channel2,
        }
    }
}
