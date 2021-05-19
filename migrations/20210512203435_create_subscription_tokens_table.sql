CREATE TABLE subscription_tokens
(
    subscription_token TEXT NOT NULL UNIQUE,
    subscriber_id      uuid NOT NULL REFERENCES subscriptions (id)
);