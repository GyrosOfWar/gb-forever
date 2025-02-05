CREATE TABLE playlist_entry (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    video_id BIGINT NOT NULL REFERENCES gb_videos (id) UNIQUE,
    "status" VARCHAR NOT NULL,
    file_path VARCHAR,
    last_progress INT
);
