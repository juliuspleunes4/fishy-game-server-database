CREATE TABLE users (
    user_id UUID PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password TEXT NOT NULL,
    salt TEXT NOT NULL,
    created TIMESTAMPTZ NOT NULL
);

CREATE TABLE friends (
    user_one_id UUID NOT NULL,
    user_two_id UUID NOT NULL,
    CONSTRAINT friend_order CHECK (user_one_id < user_two_id), -- Also makes sure that a player is not befriend with himself
    CONSTRAINT unique_friend UNIQUE (user_one_id, user_two_id)
);

CREATE TABLE friend_requests (
    user_one_id UUID NOT NULL,
    user_two_id UUID NOT NULL,
    request_sender_id UUID NOT NULL,
    request_created_time TIMESTAMPTZ NOT NULL
    CONSTRAINT user_order CHECK (user_one_id < user_two_id), -- Also makes sure that a player is not requesting to befriend himself
    CONSTRAINT sender_in_pair CHECK (request_sender_id = user_one_id OR request_sender_id = user_two_id),
    CONSTRAINT unique_friend_request UNIQUE (user_one_id, user_two_id)
);

CREATE TABLE stats (
    user_id UUID PRIMARY KEY REFERENCES users(user_id),
    selected_rod UUID,
    selected_bait UUID,
    xp INTEGER NOT NULL,
    coins INTEGER NOT NULL,
    bucks INTEGER NOT NULL,
    total_playtime INTEGER NOT NULL
);

CREATE TABLE fish_caught (
    user_id UUID NOT NULL REFERENCES stats(user_id),
    fish_id INT NOT NULL,
    amount INT NOT NULL,
    max_length INTEGER NOT NULL,
    first_caught DATE NOT NULL,
    PRIMARY KEY (user_id, fish_id)
);

CREATE TABLE fish_caught_area (
    user_id UUID NOT null,
    fish_id INTEGER NOT NULL,
    area_id INTEGER NOT NULL,
    PRIMARY KEY (user_id, fish_id, area_id) ,
    FOREIGN KEY (user_id, fish_id) REFERENCES fish_caught(user_id, fish_id)
);

CREATE TABLE fish_caught_bait (
    user_id UUID NOT null,
    fish_id INTEGER NOT NULL,
    bait_id INTEGER NOT NULL,
    PRIMARY KEY (user_id, fish_id, bait_id),
    FOREIGN KEY (user_id, fish_id) REFERENCES fish_caught(user_id, fish_id)
);

CREATE TABLE inventory_item (
    user_id UUID NOT NULL REFERENCES users(user_id),
    item_uuid UUID UNIQUE,
    definition_id INTEGER NOT NULL,
    state_blob TEXT NOT NULL,
    PRIMARY KEY (user_id, item_uuid)
);

CREATE TABLE mail (
    mail_id UUID PRIMARY KEY,
    sender_id UUID NOT NULL REFERENCES users(user_id),
    title TEXT NOT NULL,
    message TEXT NOT NULL,
    send_time TIMESTAMPTZ NOT NULL
);

CREATE TABLE mailbox (
    user_id UUID NOT NULL,
    mail_id UUID NOT NULL,
    read BOOLEAN NOT NULL DEFAULT FALSE,
    archived BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (user_id, mail_id),
    FOREIGN KEY (user_id) REFERENCES users(user_id),
    FOREIGN KEY (mail_id) REFERENCES mail(mail_id)
);

CREATE TABLE player_effects (
    user_id UUID NOT NULL REFERENCES users(user_id),
    item_id INTEGER NOT NULL,
    expiry_time TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (user_id, item_id),
    CONSTRAINT valid_expiry_time CHECK (expiry_time > NOW())
);

CREATE INDEX idx_player_effects_expiry ON player_effects(expiry_time);
CREATE INDEX idx_player_effects_user_id ON player_effects(user_id);

-- Competition types: 1=MostFishCompetition, 2=MostItemsCompetition, 3=LargestFishCompetition
CREATE TABLE competitions (
    competition_id UUID PRIMARY KEY,
    competition_type INTEGER NOT NULL,
    target_fish_id INTEGER NOT NULL,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    reward_currency TEXT NOT NULL, -- 'coins' or 'bucks'
    prize_1st INTEGER NOT NULL,
    prize_2nd INTEGER NOT NULL,
    prize_3rd INTEGER NOT NULL,
    prize_4th INTEGER NOT NULL,
    prize_5th INTEGER NOT NULL,
    prize_6th INTEGER NOT NULL,
    prize_7th INTEGER NOT NULL,
    prize_8th INTEGER NOT NULL,
    prize_9th INTEGER NOT NULL,
    prize_10th INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT valid_competition_type CHECK (competition_type IN (1, 2, 3)),
    CONSTRAINT valid_currency CHECK (reward_currency IN ('coins', 'bucks')),
    CONSTRAINT valid_times CHECK (start_time < end_time)
);

CREATE TABLE competition_participants (
    competition_id UUID NOT NULL REFERENCES competitions(competition_id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(user_id),
    user_name TEXT NOT NULL,
    score INTEGER NOT NULL DEFAULT 0,
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (competition_id, user_id)
);

CREATE INDEX idx_competitions_time_range ON competitions(start_time, end_time);
CREATE INDEX idx_competitions_active ON competitions(start_time, end_time) WHERE end_time > NOW();
CREATE INDEX idx_competition_participants_score ON competition_participants(competition_id, score DESC);
