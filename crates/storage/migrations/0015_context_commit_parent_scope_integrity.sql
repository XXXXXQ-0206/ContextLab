DO $parent_scope_preflight$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM context_commit_parents AS parent_link
        JOIN context_commits AS child_commit
          ON child_commit.id = parent_link.commit_id
        JOIN context_commits AS parent_commit
          ON parent_commit.id = parent_link.parent_commit_id
        WHERE child_commit.context_id <> parent_commit.context_id
    ) THEN
        RAISE EXCEPTION USING
            ERRCODE = '23503',
            CONSTRAINT = 'fk_context_commit_parents_parent_same_context',
            MESSAGE = 'context commit parent history crosses Context scope';
    END IF;
END
$parent_scope_preflight$;

ALTER TABLE context_commit_parents
    ADD COLUMN context_id UUID;

UPDATE context_commit_parents AS parent_link
SET context_id = child_commit.context_id
FROM context_commits AS child_commit
WHERE child_commit.id = parent_link.commit_id;

ALTER TABLE context_commit_parents
    ALTER COLUMN context_id SET NOT NULL;

ALTER TABLE context_commit_parents
    ADD CONSTRAINT fk_context_commit_parents_child_same_context
    FOREIGN KEY (context_id, commit_id)
    REFERENCES context_commits (context_id, id)
    ON DELETE CASCADE
    DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE context_commit_parents
    ADD CONSTRAINT fk_context_commit_parents_parent_same_context
    FOREIGN KEY (context_id, parent_commit_id)
    REFERENCES context_commits (context_id, id)
    ON DELETE RESTRICT
    DEFERRABLE INITIALLY IMMEDIATE;
