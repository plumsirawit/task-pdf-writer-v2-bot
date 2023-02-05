CREATE TABLE IF NOT EXISTS contests (
    guild_id VARCHAR(255) NOT NULL,
    git_remote_url TEXT NOT NULL,
  	contest_rel_path TEXT NOT NULL,
  	private_key BYTEA,
    PRIMARY KEY (guild_id)
);
