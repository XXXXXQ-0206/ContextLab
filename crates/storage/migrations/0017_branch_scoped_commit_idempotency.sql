ALTER TABLE context_commit_idempotency
    ADD COLUMN branch_name TEXT COLLATE "C";

UPDATE context_commit_idempotency AS receipt
SET branch_name = commit.branch_name
FROM context_commits AS commit
WHERE commit.id = receipt.commit_id
  AND commit.context_id = receipt.context_id;

ALTER TABLE context_commit_idempotency
    ALTER COLUMN branch_name SET NOT NULL;

ALTER TABLE context_commit_idempotency
    ADD CONSTRAINT chk_context_commit_idempotency_branch_name
    CHECK (
        octet_length(branch_name) > 0
        AND branch_name = btrim(branch_name)
        AND branch_name !~ '^[[:space:]]|[[:space:]]$'
        AND branch_name !~ '[[:cntrl:]]'
    );

ALTER TABLE context_commit_idempotency
    DROP CONSTRAINT context_commit_idempotency_pkey;

ALTER TABLE context_commit_idempotency
    ADD PRIMARY KEY (
        identity_source,
        principal_id,
        context_id,
        branch_name,
        idempotency_key
    );
