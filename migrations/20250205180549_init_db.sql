CREATE TABLE gb_videos (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    "date" VARCHAR,
    "description" VARCHAR,
    title VARCHAR NOT NULL,
    item_size BIGINT,
    identifier VARCHAR NOT NULL UNIQUE,
    external_identifier VARCHAR,
    collections VARCHAR[],
    creator VARCHAR
);

CREATE TABLE playlist_entry (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    video_id BIGINT NOT NULL REFERENCES gb_videos (id) UNIQUE,
    "status" VARCHAR NOT NULL,
    file_path VARCHAR,
    last_progress INT
);

CREATE TABLE active_playlist_entry (
    id BIGINT NOT NULL,
    entry_index BIGINT NOT NULL REFERENCES playlist_entry (id)
);