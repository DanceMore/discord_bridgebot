CREATE TABLE channel_pairs (
  id SERIAL PRIMARY KEY,
  channel1 bigint NOT NULL,
  channel2 bigint NOT NULL,
  CONSTRAINT unique_channel_pair UNIQUE (channel1, channel2)
);
