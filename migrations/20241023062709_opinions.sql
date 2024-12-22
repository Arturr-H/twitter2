CREATE TABLE post_opinions (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    post_id BIGINT REFERENCES posts(id) ON DELETE CASCADE NOT NULL,

    votes INT NOT NULL DEFAULT 0,
    opinion VARCHAR(12) NOT NULL,

    UNIQUE(opinion, post_id)
);

CREATE TABLE post_opinion_votes (
    opinion_id BIGINT REFERENCES post_opinions(id) ON DELETE CASCADE,
    user_id BIGINT REFERENCES users(id) ON DELETE CASCADE,
    post_id BIGINT REFERENCES posts(id) ON DELETE CASCADE,

    PRIMARY KEY (user_id, post_id, opinion_id)
);
