-- Your SQL goes here
CREATE TABLE IF NOT EXISTS subscription_group(
    id VARCHAR NOT NULL PRIMARY KEY,
    account_id VARCHAR NOT NULL REFERENCES account(id),
    title VARCHAR NOT NULL
);
CREATE TABLE IF NOT EXISTS subscription_group_member(
    subscription_group_id VARCHAR NOT NULL REFERENCES subscription_group(id),
    channel_id VARCHAR NOT NULL REFERENCES channel(id),
    PRIMARY KEY(
        subscription_group_id,
        channel_id
    )
)