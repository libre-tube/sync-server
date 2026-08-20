-- Your SQL goes here
CREATE TABLE IF NOT EXISTS subscription_group(
    id VARCHAR NOT NULL PRIMARY KEY,
    account_id VARCHAR NOT NULL CONSTRAINT FK__subscription_group__account REFERENCES account(id) ON DELETE CASCADE,
    title VARCHAR NOT NULL
);
CREATE TABLE IF NOT EXISTS subscription_group_member(
    subscription_group_id VARCHAR NOT NULL CONSTRAINT FK__subscription_group_member__subscription_group REFERENCES subscription_group(id) ON DELETE CASCADE,
    channel_id VARCHAR NOT NULL CONSTRAINT FK__subscription_group_member__channel REFERENCES channel(id) ON DELETE RESTRICT,
    PRIMARY KEY(
        subscription_group_id,
        channel_id
    )
)
