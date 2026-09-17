ALTER TABLE public.protected_route_rate_limit_configurations
    RENAME COLUMN max_tracked_principals TO max_tracked_keys;

ALTER TABLE public.protected_route_rate_limit_configurations
    DROP COLUMN last_observed_at;

CREATE FUNCTION public.prevent_protected_route_rate_limit_configuration_mutation()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    RAISE EXCEPTION 'protected route rate-limit configuration is immutable';
END;
$$;

CREATE TRIGGER protected_route_rate_limit_configurations_append_only
    BEFORE UPDATE OR DELETE ON public.protected_route_rate_limit_configurations
    FOR EACH ROW
    EXECUTE FUNCTION public.prevent_protected_route_rate_limit_configuration_mutation();
