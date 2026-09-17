ALTER TABLE benchmark_decision_evidence
    ADD CONSTRAINT uq_benchmark_decision_evidence_digest
    UNIQUE (project_id, context_id, context_commit_id, decision_id, evidence_digest);

CREATE TABLE benchmark_workspace_projection_receipts (
    project_id UUID NOT NULL,
    context_id UUID NOT NULL,
    context_commit_id UUID NOT NULL,
    cohort_id UUID NOT NULL,
    decision_id UUID NOT NULL,
    receipt_schema_version SMALLINT NOT NULL CHECK (receipt_schema_version = 1),
    evidence_digest TEXT NOT NULL CHECK (evidence_digest ~ '^sha256:[0-9a-f]{64}$'),
    case_count INTEGER NOT NULL CHECK (case_count > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (cohort_id),
    UNIQUE (project_id, context_id, context_commit_id, decision_id),
    UNIQUE (project_id, context_id, context_commit_id, decision_id, cohort_id),
    FOREIGN KEY (project_id, context_id, context_commit_id, decision_id)
        REFERENCES benchmark_decision_seals(project_id, context_id, context_commit_id, decision_id)
        ON DELETE RESTRICT,
    FOREIGN KEY (project_id, context_id, context_commit_id, decision_id, evidence_digest)
        REFERENCES benchmark_decision_evidence(
            project_id,
            context_id,
            context_commit_id,
            decision_id,
            evidence_digest
        ) ON DELETE RESTRICT
);

CREATE TABLE benchmark_workspace_projection_cases (
    project_id UUID NOT NULL,
    context_id UUID NOT NULL,
    context_commit_id UUID NOT NULL,
    decision_id UUID NOT NULL,
    cohort_id UUID NOT NULL,
    position INTEGER NOT NULL CHECK (position >= 0),
    dataset_id UUID NOT NULL,
    case_id UUID NOT NULL,
    run_id UUID NOT NULL,
    PRIMARY KEY (cohort_id, dataset_id, case_id),
    UNIQUE (cohort_id, position),
    UNIQUE (cohort_id, run_id),
    FOREIGN KEY (project_id, context_id, context_commit_id, decision_id, cohort_id)
        REFERENCES benchmark_workspace_projection_receipts(
            project_id,
            context_id,
            context_commit_id,
            decision_id,
            cohort_id
        ) ON DELETE RESTRICT,
    FOREIGN KEY (project_id, dataset_id, case_id)
        REFERENCES benchmark_dataset_cases(project_id, dataset_id, case_id) ON DELETE RESTRICT,
    FOREIGN KEY (project_id, context_id, context_commit_id, decision_id, run_id)
        REFERENCES benchmark_decision_runs(
            project_id,
            context_id,
            context_commit_id,
            decision_id,
            run_id
        ) ON DELETE RESTRICT
);

CREATE TABLE benchmark_workspace_projection_receipt_seals (
    cohort_id UUID PRIMARY KEY,
    sealed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (cohort_id)
        REFERENCES benchmark_workspace_projection_receipts(cohort_id) ON DELETE RESTRICT
);

CREATE FUNCTION validate_benchmark_workspace_projection_complete(target_cohort_id UUID)
RETURNS VOID LANGUAGE plpgsql AS $$
DECLARE
    expected_case_count INTEGER;
    actual_case_count INTEGER;
    minimum_position INTEGER;
    maximum_position INTEGER;
BEGIN
    SELECT case_count
    INTO expected_case_count
    FROM benchmark_workspace_projection_receipts
    WHERE cohort_id = target_cohort_id;

    IF expected_case_count IS NULL THEN
        RAISE EXCEPTION 'benchmark workspace projection receipt is missing';
    END IF;

    SELECT count(*)::INTEGER, min(position), max(position)
    INTO actual_case_count, minimum_position, maximum_position
    FROM benchmark_workspace_projection_cases
    WHERE cohort_id = target_cohort_id;

    IF actual_case_count <> expected_case_count
       OR minimum_position <> 0
       OR maximum_position <> expected_case_count - 1 THEN
        RAISE EXCEPTION 'benchmark workspace projection receipt is incomplete';
    END IF;

    IF EXISTS (
        (
            SELECT suite_dataset.dataset_id, dataset_case.case_id
            FROM benchmark_workspace_projection_receipts AS receipt
            INNER JOIN benchmark_decision_evidence AS evidence
              ON evidence.project_id = receipt.project_id
             AND evidence.context_id = receipt.context_id
             AND evidence.context_commit_id = receipt.context_commit_id
             AND evidence.decision_id = receipt.decision_id
            INNER JOIN benchmark_suite_datasets AS suite_dataset
              ON suite_dataset.project_id = evidence.project_id
             AND suite_dataset.suite_id = evidence.suite_id
            INNER JOIN benchmark_dataset_cases AS dataset_case
              ON dataset_case.project_id = suite_dataset.project_id
             AND dataset_case.dataset_id = suite_dataset.dataset_id
            WHERE receipt.cohort_id = target_cohort_id
            EXCEPT
            SELECT projection_case.dataset_id, projection_case.case_id
            FROM benchmark_workspace_projection_cases AS projection_case
            WHERE projection_case.cohort_id = target_cohort_id
        )
        UNION ALL
        (
            SELECT projection_case.dataset_id, projection_case.case_id
            FROM benchmark_workspace_projection_cases AS projection_case
            WHERE projection_case.cohort_id = target_cohort_id
            EXCEPT
            SELECT suite_dataset.dataset_id, dataset_case.case_id
            FROM benchmark_workspace_projection_receipts AS receipt
            INNER JOIN benchmark_decision_evidence AS evidence
              ON evidence.project_id = receipt.project_id
             AND evidence.context_id = receipt.context_id
             AND evidence.context_commit_id = receipt.context_commit_id
             AND evidence.decision_id = receipt.decision_id
            INNER JOIN benchmark_suite_datasets AS suite_dataset
              ON suite_dataset.project_id = evidence.project_id
             AND suite_dataset.suite_id = evidence.suite_id
            INNER JOIN benchmark_dataset_cases AS dataset_case
              ON dataset_case.project_id = suite_dataset.project_id
             AND dataset_case.dataset_id = suite_dataset.dataset_id
            WHERE receipt.cohort_id = target_cohort_id
        )
    ) THEN
        RAISE EXCEPTION 'benchmark workspace projection case provenance is incomplete';
    END IF;

    IF EXISTS (
        (
            SELECT decision_run.run_id
            FROM benchmark_workspace_projection_receipts AS receipt
            INNER JOIN benchmark_decision_runs AS decision_run
              ON decision_run.project_id = receipt.project_id
             AND decision_run.context_id = receipt.context_id
             AND decision_run.context_commit_id = receipt.context_commit_id
             AND decision_run.decision_id = receipt.decision_id
            WHERE receipt.cohort_id = target_cohort_id
            EXCEPT
            SELECT projection_case.run_id
            FROM benchmark_workspace_projection_cases AS projection_case
            WHERE projection_case.cohort_id = target_cohort_id
        )
        UNION ALL
        (
            SELECT projection_case.run_id
            FROM benchmark_workspace_projection_cases AS projection_case
            WHERE projection_case.cohort_id = target_cohort_id
            EXCEPT
            SELECT decision_run.run_id
            FROM benchmark_workspace_projection_receipts AS receipt
            INNER JOIN benchmark_decision_runs AS decision_run
              ON decision_run.project_id = receipt.project_id
             AND decision_run.context_id = receipt.context_id
             AND decision_run.context_commit_id = receipt.context_commit_id
             AND decision_run.decision_id = receipt.decision_id
            WHERE receipt.cohort_id = target_cohort_id
        )
    ) THEN
        RAISE EXCEPTION 'benchmark workspace projection run provenance is incomplete';
    END IF;
END;
$$;

CREATE FUNCTION seal_complete_benchmark_workspace_projection()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    PERFORM validate_benchmark_workspace_projection_complete(NEW.cohort_id);
    INSERT INTO benchmark_workspace_projection_receipt_seals (cohort_id)
    VALUES (NEW.cohort_id)
    ON CONFLICT (cohort_id) DO NOTHING;
    RETURN NEW;
END;
$$;

CREATE FUNCTION validate_benchmark_workspace_projection_seal()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    PERFORM validate_benchmark_workspace_projection_complete(NEW.cohort_id);
    RETURN NEW;
END;
$$;

CREATE FUNCTION prevent_benchmark_workspace_projection_mutation()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'benchmark workspace projection receipts are append-only';
END;
$$;

CREATE FUNCTION prevent_benchmark_workspace_projection_case_insert_after_seal()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM benchmark_workspace_projection_receipt_seals
        WHERE cohort_id = NEW.cohort_id
    ) THEN
        RAISE EXCEPTION 'benchmark workspace projection receipt is sealed';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER benchmark_workspace_projection_receipts_append_only
BEFORE UPDATE OR DELETE ON benchmark_workspace_projection_receipts
FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_workspace_projection_mutation();

CREATE TRIGGER benchmark_workspace_projection_cases_append_only
BEFORE UPDATE OR DELETE ON benchmark_workspace_projection_cases
FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_workspace_projection_mutation();

CREATE TRIGGER benchmark_workspace_projection_receipt_seals_append_only
BEFORE UPDATE OR DELETE ON benchmark_workspace_projection_receipt_seals
FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_workspace_projection_mutation();

CREATE TRIGGER benchmark_workspace_projection_cases_reject_after_seal
BEFORE INSERT ON benchmark_workspace_projection_cases
FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_workspace_projection_case_insert_after_seal();

CREATE TRIGGER benchmark_workspace_projection_seals_validate_complete
BEFORE INSERT ON benchmark_workspace_projection_receipt_seals
FOR EACH ROW EXECUTE FUNCTION validate_benchmark_workspace_projection_seal();

CREATE CONSTRAINT TRIGGER benchmark_workspace_projection_receipts_seal_complete
AFTER INSERT ON benchmark_workspace_projection_receipts
DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW EXECUTE FUNCTION seal_complete_benchmark_workspace_projection();

CREATE CONSTRAINT TRIGGER benchmark_workspace_projection_cases_seal_complete
AFTER INSERT ON benchmark_workspace_projection_cases
DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW EXECUTE FUNCTION seal_complete_benchmark_workspace_projection();
