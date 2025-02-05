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