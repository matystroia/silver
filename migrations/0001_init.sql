CREATE TABLE directories (
    path TEXT PRIMARY KEY
) STRICT;

CREATE TABLE genres (
    genre TEXT PRIMARY KEY
) STRICT;

CREATE TABLE movies (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    original_title TEXT NOT NULL,
    original_language TEXT NOT NULL,
    release_date TEXT NOT NULL,
    tagline TEXT NOT NULL,
    overview TEXT NOT NULL
) STRICT;

CREATE TABLE movie_genres (
    movie_id INTEGER NOT NULL REFERENCES movies (id) ON DELETE CASCADE,
    genre TEXT NOT NULL REFERENCES genres (genre) ON DELETE RESTRICT,
    PRIMARY KEY (movie_id, genre)
) STRICT;

CREATE TABLE series (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    original_name TEXT NOT NULL,
    original_language TEXT NOT NULL,
    tagline TEXT NOT NULL,
    overview TEXT NOT NULL
) STRICT;

CREATE TABLE series_genres (
    series_id INTEGER NOT NULL REFERENCES series (id) ON DELETE CASCADE,
    genre TEXT NOT NULL REFERENCES genres (genre) ON DELETE RESTRICT,
    PRIMARY KEY (series_id, genre)
);

CREATE TABLE seasons (
    id INTEGER PRIMARY KEY,
    series_id INTEGER NOT NULL REFERENCES series (id) ON DELETE CASCADE,
    number INTEGER NOT NULL,
    name TEXT NOT NULL
) STRICT;

CREATE TABLE episodes (
    id INTEGER PRIMARY KEY,
    series_id INTEGER NOT NULL REFERENCES series (id) ON DELETE CASCADE,
    season_number INTEGER NOT NULL,
    episode_number INTEGER NOT NULL,
    title TEXT NOT NULL,
    overview TEXT NOT NULL,
    release_date TEXT NOT NULL
) STRICT;

CREATE TABLE files (
    path TEXT PRIMARY KEY,
    hash TEXT NOT NULL,
    scanned INTEGER NOT NULL DEFAULT 0 CHECK (scanned IN (0, 1))
) STRICT;

CREATE TABLE features (
    path TEXT PRIMARY KEY REFERENCES files (path) ON DELETE CASCADE,
    movie_id INTEGER REFERENCES movies (id) ON DELETE CASCADE,
    episode_id INTEGER REFERENCES episodes (id) ON DELETE CASCADE,
    CHECK ((movie_id IS NULL) <> (episode_id IS NULL))
) STRICT;

CREATE UNIQUE INDEX idx_seasons_coords ON seasons (series_id, number);
CREATE UNIQUE INDEX idx_episodes_coords ON episodes (
    id, season_number, episode_number
);

CREATE INDEX idx_features_movie_id ON features (movie_id);
CREATE INDEX idx_features_episode_id ON features (episode_id);
