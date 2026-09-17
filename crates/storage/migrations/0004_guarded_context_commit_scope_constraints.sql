ALTER TABLE context_commits
    ADD CONSTRAINT uq_context_commits_context_id_id UNIQUE (context_id, id);

ALTER TABLE context_branches
    ADD CONSTRAINT fk_context_branches_head_commit_same_context
    FOREIGN KEY (context_id, head_commit_id)
    REFERENCES context_commits (context_id, id)
    DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE context_commit_idempotency
    ADD CONSTRAINT fk_context_commit_idempotency_same_context
    FOREIGN KEY (context_id, commit_id)
    REFERENCES context_commits (context_id, id)
    DEFERRABLE INITIALLY IMMEDIATE;
