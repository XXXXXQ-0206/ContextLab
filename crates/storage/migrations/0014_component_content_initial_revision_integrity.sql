LOCK TABLE context_components,
           context_component_content_revisions
    IN SHARE ROW EXCLUSIVE MODE;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM context_component_content_revisions AS revision
        JOIN context_components AS component
          ON component.context_id = revision.context_id
         AND component.id = revision.component_id
        WHERE revision.previous_content_hash IS NULL
          AND (
              revision.created_at IS DISTINCT FROM component.created_at
              OR EXISTS (
                  SELECT 1
                  FROM context_component_content_revisions AS other_revision
                  WHERE other_revision.context_id = revision.context_id
                    AND other_revision.component_id = revision.component_id
                    AND other_revision.commit_id <> revision.commit_id
              )
          )
    ) THEN
        RAISE EXCEPTION
            'null previous_content_hash is valid only for a component initial revision';
    END IF;
END;
$$;

CREATE UNIQUE INDEX uq_context_component_content_revisions_initial_prior
    ON context_component_content_revisions (context_id, component_id)
    WHERE previous_content_hash IS NULL;

CREATE FUNCTION validate_context_component_initial_content_revision()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
DECLARE
    component_created_at TIMESTAMPTZ;
BEGIN
    IF NEW.previous_content_hash IS NULL THEN
        SELECT created_at
        INTO component_created_at
        FROM context_components
        WHERE context_id = NEW.context_id
          AND id = NEW.component_id
        FOR UPDATE;

        IF NOT FOUND THEN
            RAISE EXCEPTION
                'component content revision must reference an existing component'
                USING ERRCODE = '23503';
        END IF;

        IF NEW.created_at IS DISTINCT FROM component_created_at THEN
            RAISE EXCEPTION
                'null previous_content_hash must use the component creation timestamp'
                USING ERRCODE = '23514';
        END IF;

        IF EXISTS (
            SELECT 1
            FROM context_component_content_revisions
            WHERE context_id = NEW.context_id
              AND component_id = NEW.component_id
              AND commit_id <> NEW.commit_id
        ) THEN
            RAISE EXCEPTION
                'null previous_content_hash is valid only for a component initial revision'
                USING ERRCODE = '23514';
        END IF;
    END IF;

    RETURN NEW;
END;
$$;

CREATE TRIGGER context_component_content_revisions_initial_revision_integrity
    BEFORE INSERT OR UPDATE OF context_id, component_id, previous_content_hash, created_at
    ON context_component_content_revisions
    FOR EACH ROW
    EXECUTE FUNCTION validate_context_component_initial_content_revision();
