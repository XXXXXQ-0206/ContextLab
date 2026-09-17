CREATE TABLE public.protected_route_rate_limit_configurations (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    max_requests INTEGER NOT NULL CHECK (max_requests BETWEEN 1 AND 1000),
    window_seconds INTEGER NOT NULL CHECK (window_seconds BETWEEN 1 AND 3600),
    max_tracked_principals INTEGER NOT NULL CHECK (max_tracked_principals BETWEEN 1 AND 100000),
    last_observed_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE public.protected_route_rate_limit_states (
    identity_source TEXT COLLATE "C" NOT NULL
        CHECK (
            octet_length(identity_source) BETWEEN 1 AND 2048
            AND identity_source !~ '^[[:space:]]|[[:space:]]$'
        ),
    principal_id TEXT COLLATE "C" NOT NULL
        CHECK (
            octet_length(principal_id) BETWEEN 1 AND 512
            AND principal_id !~ '^[[:space:]]|[[:space:]]$'
        ),
    operation TEXT COLLATE "C" NOT NULL
        CHECK (
            octet_length(operation) BETWEEN 1 AND 128
            AND operation !~ '^[[:space:]]|[[:space:]]$'
        ),
    request_timestamps TIMESTAMPTZ[] NOT NULL
        CHECK (cardinality(request_timestamps) BETWEEN 1 AND 1000),
    expires_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (identity_source, principal_id, operation)
);

CREATE INDEX idx_protected_route_rate_limit_states_expires_at
    ON public.protected_route_rate_limit_states(expires_at);
