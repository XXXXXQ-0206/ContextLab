CREATE TABLE public.context_authorization_audit_purge_manifest_items (
    manifest_id UUID NOT NULL REFERENCES public.context_authorization_audit_purge_manifests(id) ON DELETE RESTRICT,
    event_id UUID NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL,
    context_id UUID NOT NULL,
    permission TEXT NOT NULL CHECK (permission IN ('read', 'write')),
    decision TEXT NOT NULL CHECK (decision IN ('granted', 'forbidden', 'unavailable')),
    PRIMARY KEY (manifest_id, event_id),
    UNIQUE (event_id)
);

REVOKE ALL PRIVILEGES ON TABLE public.context_authorization_audit_purge_manifest_items
    FROM contextlab_audit_purge_owner;
GRANT SELECT, INSERT ON TABLE public.context_authorization_audit_purge_manifest_items
    TO contextlab_audit_purge_owner;
REVOKE ALL PRIVILEGES ON TABLE public.context_authorization_audit_purge_manifest_items
    FROM contextlab_audit_purge_executor;

CREATE OR REPLACE FUNCTION public.prevent_context_authorization_audit_purge_manifest_item_mutation()
RETURNS TRIGGER
LANGUAGE plpgsql
SET search_path = pg_catalog, public
AS $$
BEGIN
    RAISE EXCEPTION 'context authorization audit purge manifest items are append-only';
END;
$$;

DROP TRIGGER IF EXISTS context_authorization_audit_purge_manifest_items_append_only
    ON public.context_authorization_audit_purge_manifest_items;
CREATE TRIGGER context_authorization_audit_purge_manifest_items_append_only
    BEFORE UPDATE OR DELETE ON public.context_authorization_audit_purge_manifest_items
    FOR EACH ROW
    EXECUTE FUNCTION public.prevent_context_authorization_audit_purge_manifest_item_mutation();

CREATE OR REPLACE FUNCTION public.prevent_context_authorization_audit_event_mutation()
RETURNS TRIGGER
LANGUAGE plpgsql
SET search_path = pg_catalog, public
AS $$
DECLARE
    manifest_marker TEXT;
BEGIN
    manifest_marker := current_setting('contextlab.audit_purge_manifest_id', true);

    IF TG_OP = 'DELETE'
        AND current_user = 'contextlab_audit_purge_owner'
        AND manifest_marker IS NOT NULL
        AND manifest_marker <> ''
        AND EXISTS (
            SELECT 1
            FROM public.context_authorization_audit_purge_manifest_items
            WHERE manifest_id::TEXT = manifest_marker
                AND event_id = OLD.id
        ) THEN
        RETURN OLD;
    END IF;

    RAISE EXCEPTION 'context authorization audit events are append-only';
END;
$$;

DROP TRIGGER IF EXISTS context_authorization_audit_events_append_only
    ON public.context_authorization_audit_events;
CREATE TRIGGER context_authorization_audit_events_append_only
    BEFORE UPDATE OR DELETE ON public.context_authorization_audit_events
    FOR EACH ROW
    EXECUTE FUNCTION public.prevent_context_authorization_audit_event_mutation();

CREATE OR REPLACE FUNCTION public.purge_context_authorization_audit_events(
    p_workspace_id UUID,
    p_policy_revision_id UUID,
    p_cutoff TIMESTAMPTZ,
    p_limit INTEGER DEFAULT 100
) RETURNS TABLE (manifest_id UUID, purged_event_count BIGINT)
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog, public
AS $$
DECLARE
    selected_event_ids UUID[];
    selected_event_count BIGINT;
    created_manifest_id UUID;
BEGIN
    IF p_workspace_id IS NULL OR p_policy_revision_id IS NULL OR p_cutoff IS NULL THEN
        RAISE EXCEPTION 'workspace, policy revision, and cutoff are required';
    END IF;

    IF p_limit IS NULL OR p_limit <= 0 OR p_limit > 1000 THEN
        RAISE EXCEPTION 'purge limit must be between 1 and 1000';
    END IF;

    PERFORM 1
    FROM public.context_authorization_audit_retention_policies
    WHERE workspace_id = p_workspace_id
        AND revision_id = p_policy_revision_id;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'authorization audit retention policy does not belong to the workspace';
    END IF;

    SELECT array_agg(eligible_events.id ORDER BY eligible_events.purge_eligible_at, eligible_events.id)
    INTO selected_event_ids
    FROM (
        SELECT audit_events.id, audit_events.purge_eligible_at
        FROM public.context_authorization_audit_events AS audit_events
        WHERE audit_events.context_id IN (
            SELECT contexts.id
            FROM public.contexts
            JOIN public.projects ON projects.id = contexts.project_id
            WHERE projects.workspace_id = p_workspace_id
        )
            AND audit_events.retention_disposition = 'purge_eligible'
            AND audit_events.retention_policy_revision_id = p_policy_revision_id
            AND audit_events.purge_eligible_at < p_cutoff
        ORDER BY audit_events.purge_eligible_at, audit_events.id
        LIMIT p_limit
        FOR UPDATE SKIP LOCKED
    ) AS eligible_events;

    selected_event_count := COALESCE(array_length(selected_event_ids, 1), 0);
    IF selected_event_count = 0 THEN
        RETURN QUERY SELECT NULL::UUID, 0::BIGINT;
        RETURN;
    END IF;

    INSERT INTO public.context_authorization_audit_purge_manifests (
        workspace_id,
        policy_revision_id,
        cutoff,
        selected_event_count
    )
    VALUES (
        p_workspace_id,
        p_policy_revision_id,
        p_cutoff,
        selected_event_count
    )
    RETURNING id INTO created_manifest_id;

    INSERT INTO public.context_authorization_audit_purge_manifest_items (
        manifest_id,
        event_id,
        recorded_at,
        context_id,
        permission,
        decision
    )
    SELECT
        created_manifest_id,
        audit_events.id,
        audit_events.recorded_at,
        audit_events.context_id,
        audit_events.permission,
        audit_events.decision
    FROM public.context_authorization_audit_events AS audit_events
    WHERE audit_events.id = ANY(selected_event_ids)
    ORDER BY audit_events.purge_eligible_at, audit_events.id;

    PERFORM set_config(
        'contextlab.audit_purge_manifest_id',
        created_manifest_id::TEXT,
        true
    );

    DELETE FROM public.context_authorization_audit_events
    WHERE id = ANY(selected_event_ids);

    RETURN QUERY SELECT created_manifest_id, selected_event_count;
END;
$$;

REVOKE ALL ON FUNCTION public.purge_context_authorization_audit_events(UUID, UUID, TIMESTAMPTZ, INTEGER)
    FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.purge_context_authorization_audit_events(UUID, UUID, TIMESTAMPTZ, INTEGER)
    TO contextlab_audit_purge_executor;
GRANT CREATE ON SCHEMA public TO contextlab_audit_purge_owner;
ALTER FUNCTION public.purge_context_authorization_audit_events(UUID, UUID, TIMESTAMPTZ, INTEGER)
    OWNER TO contextlab_audit_purge_owner;
REVOKE CREATE ON SCHEMA public FROM contextlab_audit_purge_owner;
