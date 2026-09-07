CREATE TABLE teams (
    id INTEGER PRIMARY KEY,
    abbreviation TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    city TEXT
);

CREATE TABLE players (
    id INTEGER PRIMARY KEY,
    first_name VARCHAR(30) NOT NULL,
    middle_name VARCHAR(30),
    last_name VARCHAR(30) NOT NULL,
    position VARCHAR(2),
    CHECK (position IS NULL OR position IN ('G', 'F', 'C'))
);

CREATE TABLE seasons (
    id INTEGER PRIMARY KEY,
    year INTEGER NOT NULL,
    league TEXT NOT NULL DEFAULT 'NBA',
    UNIQUE (year, league)
);

CREATE TABLE games (
    id INTEGER PRIMARY KEY,
    season_id INTEGER NOT NULL,
    game_date TEXT NOT NULL,
    home_team_id INTEGER NOT NULL,
    away_team_id INTEGER NOT NULL,
    home_score INTEGER,
    away_score INTEGER,
    FOREIGN KEY (season_id) REFERENCES seasons(id),
    FOREIGN KEY (home_team_id) REFERENCES teams(id),
    FOREIGN KEY (away_team_id) REFERENCES teams(id),
    CHECK (home_team_id <> away_team_id)
);

CREATE TABLE player_game_stats (
    player_id INTEGER NOT NULL,
    game_id INTEGER NOT NULL,
    team_id INTEGER NOT NULL,
    minutes REAL,
    points REAL,
    assists REAL,
    rebounds REAL,
    plus_minus REAL,
    PRIMARY KEY (player_id, game_id),
    FOREIGN KEY (player_id) REFERENCES players(id),
    FOREIGN KEY (game_id) REFERENCES games(id),
    FOREIGN KEY (team_id) REFERENCES teams(id),
    CHECK (minutes IS NULL OR minutes >= 0),
    CHECK (points IS NULL OR points >= 0),
    CHECK (assists IS NULL OR assists >= 0),
    CHECK (rebounds IS NULL OR rebounds >= 0)
);

CREATE INDEX idx_games_season ON games(season_id);
CREATE INDEX idx_games_home_team ON games(home_team_id);
CREATE INDEX idx_games_away_team ON games(away_team_id);
CREATE INDEX idx_player_game_stats_game ON player_game_stats(game_id);
CREATE INDEX idx_player_game_stats_team ON player_game_stats(team_id);
