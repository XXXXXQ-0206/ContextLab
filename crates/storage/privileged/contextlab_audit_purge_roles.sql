DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'contextlab_audit_purge_owner') THEN
        CREATE ROLE contextlab_audit_purge_owner NOLOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'contextlab_audit_purge_executor') THEN
        CREATE ROLE contextlab_audit_purge_executor NOLOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS;
    END IF;
END;
$$;

ALTER ROLE contextlab_audit_purge_owner NOLOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS;
ALTER ROLE contextlab_audit_purge_executor NOLOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS;
REVOKE contextlab_audit_purge_owner FROM contextlab_audit_purge_executor;

REVOKE ALL PRIVILEGES ON SCHEMA public FROM contextlab_audit_purge_owner;
REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA public FROM contextlab_audit_purge_owner;
REVOKE ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public FROM contextlab_audit_purge_owner;
GRANT USAGE ON SCHEMA public TO contextlab_audit_purge_owner;
GRANT SELECT, UPDATE, DELETE ON TABLE public.context_authorization_audit_events
    TO contextlab_audit_purge_owner;
GRANT SELECT ON TABLE public.context_authorization_audit_retention_policies,
    public.contexts,
    public.projects
    TO contextlab_audit_purge_owner;
GRANT SELECT, INSERT ON TABLE public.context_authorization_audit_purge_manifests
    TO contextlab_audit_purge_owner;

REVOKE ALL PRIVILEGES ON SCHEMA public FROM contextlab_audit_purge_executor;
REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA public FROM contextlab_audit_purge_executor;
REVOKE ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public FROM contextlab_audit_purge_executor;
GRANT USAGE ON SCHEMA public TO contextlab_audit_purge_executor;
