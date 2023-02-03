-- Add migration script here
CREATE TABLE contests (
    guild_id VARCHAR(255) NOT NULL,
    git_remote_url TEXT NOT NULL,
    PRIMARY KEY (guild_id)
)