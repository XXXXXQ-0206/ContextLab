ALTER TABLE contexts
    ADD CONSTRAINT uq_contexts_project_id_id UNIQUE (project_id, id);

CREATE TABLE benchmark_dataset_definitions (
    project_id UUID NOT NULL,
    id UUID NOT NULL,
    name TEXT NOT NULL CHECK (name = btrim(name) AND char_length(name) BETWEEN 1 AND 240),
    created_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (project_id, id),
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE RESTRICT
);

CREATE TABLE benchmark_dataset_cases (
    project_id UUID NOT NULL,
    dataset_id UUID NOT NULL,
    case_id UUID NOT NULL,
    position INTEGER NOT NULL CHECK (position >= 0),
    name TEXT NOT NULL CHECK (name = btrim(name) AND char_length(name) BETWEEN 1 AND 240),
    input JSONB NOT NULL,
    expected_mode TEXT NOT NULL CHECK (expected_mode IN ('unspecified', 'exact')),
    expected_output JSONB,
    PRIMARY KEY (project_id, dataset_id, case_id),
    UNIQUE (project_id, dataset_id, position),
    FOREIGN KEY (project_id, dataset_id)
        REFERENCES benchmark_dataset_definitions(project_id, id) ON DELETE RESTRICT,
    CHECK (
        (expected_mode = 'unspecified' AND expected_output IS NULL)
        OR (expected_mode = 'exact' AND expected_output IS NOT NULL)
    )
);

CREATE TABLE benchmark_suite_definitions (
    project_id UUID NOT NULL,
    id UUID NOT NULL,
    name TEXT NOT NULL CHECK (name = btrim(name) AND char_length(name) BETWEEN 1 AND 240),
    created_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (project_id, id),
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE RESTRICT
);

CREATE TABLE benchmark_suite_datasets (
    project_id UUID NOT NULL,
    suite_id UUID NOT NULL,
    dataset_id UUID NOT NULL,
    position INTEGER NOT NULL CHECK (position >= 0),
    PRIMARY KEY (project_id, suite_id, dataset_id),
    UNIQUE (project_id, suite_id, position),
    FOREIGN KEY (project_id, suite_id)
        REFERENCES benchmark_suite_definitions(project_id, id) ON DELETE RESTRICT,
    FOREIGN KEY (project_id, dataset_id)
        REFERENCES benchmark_dataset_definitions(project_id, id) ON DELETE RESTRICT
);

CREATE TABLE benchmark_suite_thresholds (
    project_id UUID NOT NULL,
    suite_id UUID NOT NULL,
    metric TEXT NOT NULL CHECK (metric IN (
        'latency_ms', 'cost_usd', 'accuracy', 'hallucination_rate',
        'tool_usage_count', 'token_count', 'execution_time_ms',
        'output_quality', 'success_rate'
    )),
    direction TEXT NOT NULL CHECK (direction IN ('minimum', 'maximum')),
    threshold_value DOUBLE PRECISION NOT NULL CHECK (
        threshold_value <> 'NaN'::DOUBLE PRECISION
        AND threshold_value <> 'Infinity'::DOUBLE PRECISION
        AND threshold_value <> '-Infinity'::DOUBLE PRECISION
    ),
    PRIMARY KEY (project_id, suite_id, metric),
    FOREIGN KEY (project_id, suite_id)
        REFERENCES benchmark_suite_definitions(project_id, id) ON DELETE RESTRICT
);

CREATE TABLE benchmark_evaluation_runs (
    project_id UUID NOT NULL,
    context_id UUID NOT NULL,
    context_commit_id UUID NOT NULL,
    run_id UUID NOT NULL,
    model_version TEXT NOT NULL CHECK (btrim(model_version) <> ''),
    temperature REAL NOT NULL CHECK (
        temperature <> 'NaN'::REAL
        AND temperature <> 'Infinity'::REAL
        AND temperature <> '-Infinity'::REAL
        AND temperature >= 0.0
        AND temperature <= 2.0
    ),
    executed_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (project_id, context_id, context_commit_id, run_id),
    UNIQUE (project_id, context_id, run_id),
    FOREIGN KEY (project_id, context_id) REFERENCES contexts(project_id, id) ON DELETE RESTRICT,
    FOREIGN KEY (context_id, context_commit_id)
        REFERENCES context_commits(context_id, id) ON DELETE RESTRICT
);

CREATE TABLE benchmark_run_measurements (
    project_id UUID NOT NULL,
    context_id UUID NOT NULL,
    context_commit_id UUID NOT NULL,
    run_id UUID NOT NULL,
    position INTEGER NOT NULL CHECK (position >= 0),
    metric TEXT NOT NULL CHECK (metric IN (
        'latency_ms', 'cost_usd', 'accuracy', 'hallucination_rate',
        'tool_usage_count', 'token_count', 'execution_time_ms',
        'output_quality', 'success_rate'
    )),
    value DOUBLE PRECISION NOT NULL CHECK (
        value <> 'NaN'::DOUBLE PRECISION
        AND value <> 'Infinity'::DOUBLE PRECISION
        AND value <> '-Infinity'::DOUBLE PRECISION
    ),
    PRIMARY KEY (project_id, context_id, context_commit_id, run_id, position),
    FOREIGN KEY (project_id, context_id, context_commit_id, run_id)
        REFERENCES benchmark_evaluation_runs(project_id, context_id, context_commit_id, run_id) ON DELETE RESTRICT
);

CREATE TABLE benchmark_decision_evidence (
    project_id UUID NOT NULL,
    context_id UUID NOT NULL,
    decision_id UUID NOT NULL,
    context_commit_id UUID NOT NULL,
    suite_id UUID NOT NULL,
    evaluator_key TEXT NOT NULL CHECK (btrim(evaluator_key) <> ''),
    evaluator_version TEXT NOT NULL CHECK (btrim(evaluator_version) <> ''),
    comparability_fingerprint TEXT NOT NULL CHECK (comparability_fingerprint ~ '^sha256:[0-9a-f]{64}$'),
    evidence_digest TEXT NOT NULL CHECK (evidence_digest ~ '^sha256:[0-9a-f]{64}$'),
    status TEXT NOT NULL CHECK (status IN ('passed', 'regressed', 'insufficient_data')),
    recorded_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (project_id, context_id, context_commit_id, decision_id),
    UNIQUE (project_id, context_id, context_commit_id, suite_id, decision_id),
    FOREIGN KEY (project_id, context_id) REFERENCES contexts(project_id, id) ON DELETE RESTRICT,
    FOREIGN KEY (context_id, context_commit_id)
        REFERENCES context_commits(context_id, id) ON DELETE RESTRICT,
    FOREIGN KEY (project_id, suite_id)
        REFERENCES benchmark_suite_definitions(project_id, id) ON DELETE RESTRICT
);

CREATE TABLE benchmark_decision_runs (
    project_id UUID NOT NULL,
    context_id UUID NOT NULL,
    context_commit_id UUID NOT NULL,
    decision_id UUID NOT NULL,
    run_id UUID NOT NULL,
    position INTEGER NOT NULL CHECK (position >= 0),
    PRIMARY KEY (project_id, context_id, context_commit_id, decision_id, run_id),
    UNIQUE (project_id, context_id, context_commit_id, decision_id, position),
    FOREIGN KEY (project_id, context_id, context_commit_id, decision_id)
        REFERENCES benchmark_decision_evidence(project_id, context_id, context_commit_id, decision_id) ON DELETE RESTRICT,
    FOREIGN KEY (project_id, context_id, context_commit_id, run_id)
        REFERENCES benchmark_evaluation_runs(project_id, context_id, context_commit_id, run_id) ON DELETE RESTRICT
);

CREATE TABLE benchmark_decision_metric_results (
    project_id UUID NOT NULL,
    context_id UUID NOT NULL,
    context_commit_id UUID NOT NULL,
    decision_id UUID NOT NULL,
    suite_id UUID NOT NULL,
    metric TEXT NOT NULL,
    observed_value DOUBLE PRECISION CHECK (
        observed_value IS NULL OR (
            observed_value <> 'NaN'::DOUBLE PRECISION
            AND observed_value <> 'Infinity'::DOUBLE PRECISION
            AND observed_value <> '-Infinity'::DOUBLE PRECISION
        )
    ),
    sample_count BIGINT NOT NULL CHECK (sample_count >= 0),
    required_sample_count BIGINT NOT NULL CHECK (required_sample_count >= 0),
    has_complete_coverage BOOLEAN NOT NULL,
    outcome TEXT NOT NULL CHECK (outcome IN ('passed', 'regressed', 'insufficient_data')),
    PRIMARY KEY (project_id, context_id, context_commit_id, decision_id, metric),
    FOREIGN KEY (project_id, context_id, context_commit_id, suite_id, decision_id)
        REFERENCES benchmark_decision_evidence(project_id, context_id, context_commit_id, suite_id, decision_id) ON DELETE RESTRICT,
    FOREIGN KEY (project_id, suite_id, metric)
        REFERENCES benchmark_suite_thresholds(project_id, suite_id, metric) ON DELETE RESTRICT
);

CREATE TABLE benchmark_dataset_seals (
    project_id UUID NOT NULL,
    dataset_id UUID NOT NULL,
    sealed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, dataset_id),
    FOREIGN KEY (project_id, dataset_id)
        REFERENCES benchmark_dataset_definitions(project_id, id) ON DELETE RESTRICT
);

CREATE TABLE benchmark_suite_seals (
    project_id UUID NOT NULL,
    suite_id UUID NOT NULL,
    sealed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, suite_id),
    FOREIGN KEY (project_id, suite_id)
        REFERENCES benchmark_suite_definitions(project_id, id) ON DELETE RESTRICT
);

CREATE TABLE benchmark_run_seals (
    project_id UUID NOT NULL,
    context_id UUID NOT NULL,
    context_commit_id UUID NOT NULL,
    run_id UUID NOT NULL,
    sealed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, context_id, context_commit_id, run_id),
    FOREIGN KEY (project_id, context_id, context_commit_id, run_id)
        REFERENCES benchmark_evaluation_runs(project_id, context_id, context_commit_id, run_id) ON DELETE RESTRICT
);

CREATE TABLE benchmark_decision_seals (
    project_id UUID NOT NULL,
    context_id UUID NOT NULL,
    context_commit_id UUID NOT NULL,
    decision_id UUID NOT NULL,
    sealed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, context_id, context_commit_id, decision_id),
    FOREIGN KEY (project_id, context_id, context_commit_id, decision_id)
        REFERENCES benchmark_decision_evidence(project_id, context_id, context_commit_id, decision_id) ON DELETE RESTRICT
);

CREATE INDEX idx_benchmark_dataset_definitions_project_created_at
    ON benchmark_dataset_definitions(project_id, created_at DESC, id);
CREATE INDEX idx_benchmark_suite_definitions_project_created_at
    ON benchmark_suite_definitions(project_id, created_at DESC, id);
CREATE INDEX idx_benchmark_decision_evidence_context_recorded_at
    ON benchmark_decision_evidence(context_id, recorded_at DESC, decision_id);
CREATE INDEX idx_benchmark_evaluation_runs_context_commit
    ON benchmark_evaluation_runs(project_id, context_id, context_commit_id, executed_at DESC, run_id);

CREATE FUNCTION prevent_benchmark_evidence_mutation()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'benchmark definitions, runs, and decision evidence are append-only';
END;
$$;

CREATE FUNCTION prevent_benchmark_child_insert_after_seal()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
DECLARE
    payload JSONB := to_jsonb(NEW);
BEGIN
    CASE TG_TABLE_NAME
        WHEN 'benchmark_dataset_cases' THEN
            IF EXISTS (
                SELECT 1 FROM benchmark_dataset_seals
                WHERE project_id = (payload ->> 'project_id')::UUID
                  AND dataset_id = (payload ->> 'dataset_id')::UUID
            ) THEN RAISE EXCEPTION 'benchmark dataset is sealed'; END IF;
        WHEN 'benchmark_suite_datasets', 'benchmark_suite_thresholds' THEN
            IF EXISTS (
                SELECT 1 FROM benchmark_suite_seals
                WHERE project_id = (payload ->> 'project_id')::UUID
                  AND suite_id = (payload ->> 'suite_id')::UUID
            ) THEN RAISE EXCEPTION 'benchmark suite is sealed'; END IF;
        WHEN 'benchmark_run_measurements' THEN
            IF EXISTS (
                SELECT 1 FROM benchmark_run_seals
                WHERE project_id = (payload ->> 'project_id')::UUID
                  AND context_id = (payload ->> 'context_id')::UUID
                  AND context_commit_id = (payload ->> 'context_commit_id')::UUID
                  AND run_id = (payload ->> 'run_id')::UUID
            ) THEN RAISE EXCEPTION 'benchmark run is sealed'; END IF;
        WHEN 'benchmark_decision_runs', 'benchmark_decision_metric_results' THEN
            IF EXISTS (
                SELECT 1 FROM benchmark_decision_seals
                WHERE project_id = (payload ->> 'project_id')::UUID
                  AND context_id = (payload ->> 'context_id')::UUID
                  AND context_commit_id = (payload ->> 'context_commit_id')::UUID
                  AND decision_id = (payload ->> 'decision_id')::UUID
            ) THEN RAISE EXCEPTION 'benchmark decision is sealed'; END IF;
    END CASE;
    RETURN NEW;
END;
$$;

CREATE TRIGGER benchmark_dataset_definitions_append_only BEFORE UPDATE OR DELETE ON benchmark_dataset_definitions FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_evidence_mutation();
CREATE TRIGGER benchmark_dataset_cases_append_only BEFORE UPDATE OR DELETE ON benchmark_dataset_cases FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_evidence_mutation();
CREATE TRIGGER benchmark_suite_definitions_append_only BEFORE UPDATE OR DELETE ON benchmark_suite_definitions FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_evidence_mutation();
CREATE TRIGGER benchmark_suite_datasets_append_only BEFORE UPDATE OR DELETE ON benchmark_suite_datasets FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_evidence_mutation();
CREATE TRIGGER benchmark_suite_thresholds_append_only BEFORE UPDATE OR DELETE ON benchmark_suite_thresholds FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_evidence_mutation();
CREATE TRIGGER benchmark_evaluation_runs_append_only BEFORE UPDATE OR DELETE ON benchmark_evaluation_runs FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_evidence_mutation();
CREATE TRIGGER benchmark_run_measurements_append_only BEFORE UPDATE OR DELETE ON benchmark_run_measurements FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_evidence_mutation();
CREATE TRIGGER benchmark_decision_evidence_append_only BEFORE UPDATE OR DELETE ON benchmark_decision_evidence FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_evidence_mutation();
CREATE TRIGGER benchmark_decision_runs_append_only BEFORE UPDATE OR DELETE ON benchmark_decision_runs FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_evidence_mutation();
CREATE TRIGGER benchmark_decision_metric_results_append_only BEFORE UPDATE OR DELETE ON benchmark_decision_metric_results FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_evidence_mutation();
CREATE TRIGGER benchmark_dataset_seals_append_only BEFORE UPDATE OR DELETE ON benchmark_dataset_seals FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_evidence_mutation();
CREATE TRIGGER benchmark_suite_seals_append_only BEFORE UPDATE OR DELETE ON benchmark_suite_seals FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_evidence_mutation();
CREATE TRIGGER benchmark_run_seals_append_only BEFORE UPDATE OR DELETE ON benchmark_run_seals FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_evidence_mutation();
CREATE TRIGGER benchmark_decision_seals_append_only BEFORE UPDATE OR DELETE ON benchmark_decision_seals FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_evidence_mutation();
CREATE TRIGGER benchmark_dataset_cases_reject_after_dataset_seal BEFORE INSERT ON benchmark_dataset_cases FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_child_insert_after_seal();
CREATE TRIGGER benchmark_suite_datasets_reject_after_suite_seal BEFORE INSERT ON benchmark_suite_datasets FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_child_insert_after_seal();
CREATE TRIGGER benchmark_suite_thresholds_reject_after_suite_seal BEFORE INSERT ON benchmark_suite_thresholds FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_child_insert_after_seal();
CREATE TRIGGER benchmark_run_measurements_reject_after_run_seal BEFORE INSERT ON benchmark_run_measurements FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_child_insert_after_seal();
CREATE TRIGGER benchmark_decision_runs_reject_after_decision_seal BEFORE INSERT ON benchmark_decision_runs FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_child_insert_after_seal();
CREATE TRIGGER benchmark_decision_metric_results_reject_after_decision_seal BEFORE INSERT ON benchmark_decision_metric_results FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_child_insert_after_seal();

CREATE FUNCTION validate_benchmark_dataset_definition_complete()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
DECLARE
    target_project_id UUID;
    target_dataset_id UUID;
BEGIN
    target_project_id := (to_jsonb(NEW) ->> 'project_id')::UUID;
    target_dataset_id := CASE
        WHEN TG_TABLE_NAME = 'benchmark_dataset_definitions' THEN (to_jsonb(NEW) ->> 'id')::UUID
        ELSE (to_jsonb(NEW) ->> 'dataset_id')::UUID
    END;
    IF EXISTS (
        SELECT 1 FROM benchmark_dataset_definitions
        WHERE project_id = target_project_id AND id = target_dataset_id
    ) AND NOT EXISTS (
        SELECT 1 FROM benchmark_dataset_cases
        WHERE project_id = target_project_id AND dataset_id = target_dataset_id
    ) THEN
        RAISE EXCEPTION 'benchmark dataset definition must contain at least one case';
    END IF;
    IF EXISTS (
        SELECT 1 FROM benchmark_dataset_definitions
        WHERE project_id = target_project_id AND id = target_dataset_id
    ) THEN
        INSERT INTO benchmark_dataset_seals (project_id, dataset_id)
        VALUES (target_project_id, target_dataset_id)
        ON CONFLICT DO NOTHING;
    END IF;
    RETURN NULL;
END;
$$;

CREATE CONSTRAINT TRIGGER benchmark_dataset_definition_complete
    AFTER INSERT ON benchmark_dataset_definitions
    DEFERRABLE INITIALLY DEFERRED FOR EACH ROW
    EXECUTE FUNCTION validate_benchmark_dataset_definition_complete();
CREATE CONSTRAINT TRIGGER benchmark_dataset_case_completes_definition
    AFTER INSERT ON benchmark_dataset_cases
    DEFERRABLE INITIALLY DEFERRED FOR EACH ROW
    EXECUTE FUNCTION validate_benchmark_dataset_definition_complete();

CREATE FUNCTION validate_benchmark_suite_definition_complete()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
DECLARE
    target_project_id UUID;
    target_suite_id UUID;
BEGIN
    target_project_id := (to_jsonb(NEW) ->> 'project_id')::UUID;
    target_suite_id := CASE
        WHEN TG_TABLE_NAME = 'benchmark_suite_definitions' THEN (to_jsonb(NEW) ->> 'id')::UUID
        ELSE (to_jsonb(NEW) ->> 'suite_id')::UUID
    END;
    IF EXISTS (
        SELECT 1 FROM benchmark_suite_definitions
        WHERE project_id = target_project_id AND id = target_suite_id
    ) AND (
        NOT EXISTS (
            SELECT 1 FROM benchmark_suite_datasets
            WHERE project_id = target_project_id AND suite_id = target_suite_id
        ) OR NOT EXISTS (
            SELECT 1 FROM benchmark_suite_thresholds
            WHERE project_id = target_project_id AND suite_id = target_suite_id
        )
    ) THEN
        RAISE EXCEPTION 'benchmark suite definition must contain datasets and thresholds';
    END IF;
    IF EXISTS (
        SELECT 1 FROM benchmark_suite_definitions
        WHERE project_id = target_project_id AND id = target_suite_id
    ) THEN
        INSERT INTO benchmark_suite_seals (project_id, suite_id)
        VALUES (target_project_id, target_suite_id)
        ON CONFLICT DO NOTHING;
    END IF;
    RETURN NULL;
END;
$$;

CREATE CONSTRAINT TRIGGER benchmark_suite_definition_complete
    AFTER INSERT ON benchmark_suite_definitions
    DEFERRABLE INITIALLY DEFERRED FOR EACH ROW
    EXECUTE FUNCTION validate_benchmark_suite_definition_complete();
CREATE CONSTRAINT TRIGGER benchmark_suite_dataset_completes_definition
    AFTER INSERT ON benchmark_suite_datasets
    DEFERRABLE INITIALLY DEFERRED FOR EACH ROW
    EXECUTE FUNCTION validate_benchmark_suite_definition_complete();
CREATE CONSTRAINT TRIGGER benchmark_suite_threshold_completes_definition
    AFTER INSERT ON benchmark_suite_thresholds
    DEFERRABLE INITIALLY DEFERRED FOR EACH ROW
    EXECUTE FUNCTION validate_benchmark_suite_definition_complete();

CREATE FUNCTION validate_benchmark_run_complete()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
DECLARE
    target_project_id UUID := (to_jsonb(NEW) ->> 'project_id')::UUID;
    target_context_id UUID := (to_jsonb(NEW) ->> 'context_id')::UUID;
    target_context_commit_id UUID := (to_jsonb(NEW) ->> 'context_commit_id')::UUID;
    target_run_id UUID := (to_jsonb(NEW) ->> 'run_id')::UUID;
BEGIN
    IF EXISTS (
        SELECT 1 FROM benchmark_evaluation_runs
        WHERE project_id = target_project_id
          AND context_id = target_context_id
          AND context_commit_id = target_context_commit_id
          AND run_id = target_run_id
    ) THEN
        INSERT INTO benchmark_run_seals (
            project_id, context_id, context_commit_id, run_id
        )
        VALUES (
            target_project_id, target_context_id, target_context_commit_id, target_run_id
        )
        ON CONFLICT DO NOTHING;
    END IF;
    RETURN NULL;
END;
$$;

CREATE CONSTRAINT TRIGGER benchmark_run_complete
    AFTER INSERT ON benchmark_evaluation_runs
    DEFERRABLE INITIALLY DEFERRED FOR EACH ROW
    EXECUTE FUNCTION validate_benchmark_run_complete();
CREATE CONSTRAINT TRIGGER benchmark_run_measurement_completes_run
    AFTER INSERT ON benchmark_run_measurements
    DEFERRABLE INITIALLY DEFERRED FOR EACH ROW
    EXECUTE FUNCTION validate_benchmark_run_complete();

CREATE FUNCTION validate_benchmark_decision_complete()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
DECLARE
    target_project_id UUID := (to_jsonb(NEW) ->> 'project_id')::UUID;
    target_context_id UUID := (to_jsonb(NEW) ->> 'context_id')::UUID;
    target_context_commit_id UUID := (to_jsonb(NEW) ->> 'context_commit_id')::UUID;
    target_decision_id UUID := (to_jsonb(NEW) ->> 'decision_id')::UUID;
    stored_suite_id UUID;
    run_count BIGINT;
    metric_result RECORD;
    actual_sample_count BIGINT;
    actual_complete_coverage BOOLEAN;
BEGIN
    SELECT suite_id INTO stored_suite_id
    FROM benchmark_decision_evidence
    WHERE project_id = target_project_id
      AND context_id = target_context_id
      AND context_commit_id = target_context_commit_id
      AND decision_id = target_decision_id;
    IF NOT FOUND THEN RETURN NULL; END IF;
    SELECT COUNT(*) INTO run_count
    FROM benchmark_decision_runs
    WHERE project_id = target_project_id
      AND context_id = target_context_id
      AND context_commit_id = target_context_commit_id
      AND decision_id = target_decision_id;
    IF run_count = 0 THEN
        RAISE EXCEPTION 'benchmark decision must contain run membership';
    END IF;
    IF EXISTS (
        (SELECT metric FROM benchmark_suite_thresholds
         WHERE project_id = target_project_id AND suite_id = stored_suite_id)
        EXCEPT
        (SELECT metric FROM benchmark_decision_metric_results
         WHERE project_id = target_project_id
           AND context_id = target_context_id
           AND context_commit_id = target_context_commit_id
           AND decision_id = target_decision_id)
    ) OR EXISTS (
        (SELECT metric FROM benchmark_decision_metric_results
         WHERE project_id = target_project_id
           AND context_id = target_context_id
           AND context_commit_id = target_context_commit_id
           AND decision_id = target_decision_id)
        EXCEPT
        (SELECT metric FROM benchmark_suite_thresholds
         WHERE project_id = target_project_id AND suite_id = stored_suite_id)
    ) THEN
        RAISE EXCEPTION 'benchmark decision must cover every suite threshold';
    END IF;
    FOR metric_result IN
        SELECT metric, observed_value, sample_count, required_sample_count, has_complete_coverage, outcome
        FROM benchmark_decision_metric_results
        WHERE project_id = target_project_id
          AND context_id = target_context_id
          AND context_commit_id = target_context_commit_id
          AND decision_id = target_decision_id
    LOOP
        SELECT COUNT(*) INTO actual_sample_count
        FROM benchmark_decision_runs decision_run
        WHERE decision_run.project_id = target_project_id
          AND decision_run.context_id = target_context_id
          AND decision_run.context_commit_id = target_context_commit_id
          AND decision_run.decision_id = target_decision_id
          AND EXISTS (
              SELECT 1 FROM benchmark_run_measurements measurement
              WHERE measurement.project_id = decision_run.project_id
                AND measurement.context_id = decision_run.context_id
                AND measurement.context_commit_id = decision_run.context_commit_id
                AND measurement.run_id = decision_run.run_id
                AND measurement.metric = metric_result.metric
          );
        SELECT NOT EXISTS (
            SELECT 1 FROM benchmark_decision_runs decision_run
            WHERE decision_run.project_id = target_project_id
              AND decision_run.context_id = target_context_id
              AND decision_run.context_commit_id = target_context_commit_id
              AND decision_run.decision_id = target_decision_id
              AND 1 <> (
                  SELECT COUNT(*) FROM benchmark_run_measurements measurement
                  WHERE measurement.project_id = decision_run.project_id
                    AND measurement.context_id = decision_run.context_id
                    AND measurement.context_commit_id = decision_run.context_commit_id
                    AND measurement.run_id = decision_run.run_id
                    AND measurement.metric = metric_result.metric
              )
        ) INTO actual_complete_coverage;
        IF metric_result.sample_count <> actual_sample_count
           OR metric_result.required_sample_count <> run_count
           OR metric_result.has_complete_coverage IS DISTINCT FROM actual_complete_coverage
           OR (metric_result.observed_value IS NULL) <> (actual_sample_count = 0)
        THEN
            RAISE EXCEPTION 'benchmark metric evidence does not match run coverage';
        END IF;
    END LOOP;
    INSERT INTO benchmark_decision_seals (
        project_id, context_id, context_commit_id, decision_id
    )
    VALUES (
        target_project_id, target_context_id, target_context_commit_id, target_decision_id
    )
    ON CONFLICT DO NOTHING;
    RETURN NULL;
END;
$$;

CREATE CONSTRAINT TRIGGER benchmark_decision_complete
    AFTER INSERT ON benchmark_decision_evidence
    DEFERRABLE INITIALLY DEFERRED FOR EACH ROW
    EXECUTE FUNCTION validate_benchmark_decision_complete();
CREATE CONSTRAINT TRIGGER benchmark_decision_run_completes_evidence
    AFTER INSERT ON benchmark_decision_runs
    DEFERRABLE INITIALLY DEFERRED FOR EACH ROW
    EXECUTE FUNCTION validate_benchmark_decision_complete();
CREATE CONSTRAINT TRIGGER benchmark_metric_result_completes_evidence
    AFTER INSERT ON benchmark_decision_metric_results
    DEFERRABLE INITIALLY DEFERRED FOR EACH ROW
    EXECUTE FUNCTION validate_benchmark_decision_complete();
