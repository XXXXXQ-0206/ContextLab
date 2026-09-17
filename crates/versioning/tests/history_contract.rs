#![allow(missing_docs)]

use chrono::{TimeZone, Utc};
use contextlab_context_core::ContextId;
use contextlab_versioning::{
    BranchHead, BranchName, CommitHistory, CommitId, ContextChange, ContextCommit, HistoryError,
};
use uuid::Uuid;

fn commit(
    id: u128,
    context_id: ContextId,
    branch: BranchName,
    parent_ids: Vec<CommitId>,
) -> ContextCommit {
    ContextCommit::from_persisted(
        CommitId::from_uuid(Uuid::from_u128(id)),
        context_id,
        branch,
        format!("commit-{id}"),
        parent_ids,
        vec![ContextChange::created_context("root")],
        Utc.timestamp_opt(id as i64, 0).single().expect("timestamp"),
    )
    .expect("valid commit")
}

#[test]
fn exposes_stable_branch_heads_and_root_to_head_history() {
    let context_id = ContextId::from_uuid(Uuid::from_u128(99));
    let main = BranchName::new("main").expect("branch");
    let feature = BranchName::new("feature").expect("branch");
    let root = commit(1, context_id, main.clone(), Vec::new());
    let child = commit(2, context_id, main.clone(), vec![root.id()]);
    let tip = commit(3, context_id, feature.clone(), vec![child.id()]);

    let history = CommitHistory::try_from_parts(
        context_id,
        [tip.clone(), root.clone(), child.clone()],
        [
            BranchHead::new(feature.clone(), Some(tip.id())),
            BranchHead::new(main.clone(), Some(child.id())),
        ],
    )
    .expect("valid history");

    assert_eq!(
        history.branches().collect::<Vec<_>>(),
        vec![&feature, &main]
    );
    assert_eq!(history.head(&main), Some(child.id()));
    assert_eq!(
        history
            .merge_plan(&feature, &main)
            .expect("fast-forward merge plan"),
        contextlab_versioning::MergePlan::FastForward {
            base: child.id(),
            target: tip.id(),
        }
    );
    assert_eq!(
        history
            .commits_for_branch(&feature)
            .expect("feature history")
            .iter()
            .map(|commit| commit.id())
            .collect::<Vec<_>>(),
        vec![root.id(), child.id(), tip.id()]
    );
}

#[test]
fn rejects_a_branch_head_that_is_not_a_linear_replay_tip() {
    let context_id = ContextId::new();
    let main = BranchName::default();
    let root = commit(10, context_id, main.clone(), Vec::new());
    let left = commit(11, context_id, main.clone(), vec![root.id()]);
    let right = commit(12, context_id, main.clone(), vec![root.id()]);
    let merge = commit(13, context_id, main.clone(), vec![left.id(), right.id()]);

    let history = CommitHistory::try_from_parts(
        context_id,
        [root, left, right, merge.clone()],
        [BranchHead::new(main.clone(), Some(merge.id()))],
    )
    .expect("graph is valid");

    assert!(matches!(
        history.commits_for_branch(&main),
        Err(HistoryError::NonLinearAncestry { commit_id }) if commit_id == merge.id()
    ));
}

#[test]
fn rejects_commits_outside_the_declared_context_even_when_heads_are_unborn() {
    let declared_context = ContextId::from_uuid(Uuid::from_u128(100));
    let foreign_context = ContextId::from_uuid(Uuid::from_u128(101));

    let error = CommitHistory::try_from_parts(
        declared_context,
        [commit(
            20,
            foreign_context,
            BranchName::default(),
            Vec::new(),
        )],
        [BranchHead::new(BranchName::default(), None)],
    )
    .expect_err("history scope must fail closed");

    assert!(matches!(error, HistoryError::Graph(_)));
}

#[test]
fn rejects_a_missing_parent_before_a_history_can_be_read() {
    let context_id = ContextId::new();
    let branch = BranchName::default();
    let missing_parent = CommitId::from_uuid(Uuid::from_u128(200));
    let child = commit(201, context_id, branch.clone(), vec![missing_parent]);

    let error = CommitHistory::try_from_parts(
        context_id,
        [child.clone()],
        [BranchHead::new(branch.clone(), Some(child.id()))],
    )
    .expect_err("a history must contain complete parent ancestry");

    assert!(matches!(
        error,
        HistoryError::Graph(
            contextlab_versioning::CommitGraphValidationError::UnknownParent { child: actual_child, parent }
        ) if actual_child == child.id().to_string() && parent == missing_parent.to_string()
    ));
}

#[test]
fn replays_the_same_branch_in_root_to_head_order_regardless_of_input_order() {
    let context_id = ContextId::new();
    let branch = BranchName::default();
    let root = commit(210, context_id, branch.clone(), Vec::new());
    let child = commit(211, context_id, branch.clone(), vec![root.id()]);
    let tip = commit(212, context_id, branch.clone(), vec![child.id()]);

    let first = CommitHistory::try_from_parts(
        context_id,
        [tip.clone(), root.clone(), child.clone()],
        [BranchHead::new(branch.clone(), Some(tip.id()))],
    )
    .expect("first history");
    let second = CommitHistory::try_from_parts(
        context_id,
        [child, tip.clone(), root],
        [BranchHead::new(branch.clone(), Some(tip.id()))],
    )
    .expect("second history");

    let replay_ids = |history: &CommitHistory| {
        history
            .commits_for_branch(&branch)
            .expect("linear branch history")
            .iter()
            .map(|commit| commit.id())
            .collect::<Vec<_>>()
    };

    assert_eq!(replay_ids(&first), replay_ids(&second));
    assert_eq!(
        replay_ids(&first),
        vec![
            CommitId::from_uuid(Uuid::from_u128(210)),
            CommitId::from_uuid(Uuid::from_u128(211)),
            CommitId::from_uuid(Uuid::from_u128(212)),
        ]
    );
}

#[test]
fn resolves_a_normal_first_parent_path_between_exact_commits() {
    let context_id = ContextId::new();
    let branch = BranchName::default();
    let root = commit(240, context_id, branch.clone(), Vec::new());
    let child = commit(241, context_id, branch.clone(), vec![root.id()]);
    let tip = commit(242, context_id, branch.clone(), vec![child.id()]);
    let history = CommitHistory::try_from_parts(
        context_id,
        [root.clone(), child.clone(), tip.clone()],
        [BranchHead::new(branch, Some(tip.id()))],
    )
    .expect("valid linear history");

    assert_eq!(
        history
            .normal_first_parent_path(root.id(), tip.id())
            .expect("normal first-parent path"),
        vec![root.id(), child.id(), tip.id()]
    );
}

#[test]
fn rejects_reversed_and_unrelated_normal_first_parent_ranges() {
    let context_id = ContextId::new();
    let branch = BranchName::default();
    let root = commit(250, context_id, branch.clone(), Vec::new());
    let child = commit(251, context_id, branch.clone(), vec![root.id()]);
    let unrelated = commit(252, context_id, branch.clone(), Vec::new());
    let history = CommitHistory::try_from_parts(
        context_id,
        [root.clone(), child.clone(), unrelated.clone()],
        [BranchHead::new(branch, Some(child.id()))],
    )
    .expect("valid disconnected commit graph");

    assert!(matches!(
        history.normal_first_parent_path(child.id(), root.id()),
        Err(HistoryError::InvalidNormalReplayRange {
            source_commit,
            target_commit,
        }) if source_commit == child.id() && target_commit == root.id()
    ));
    assert!(matches!(
        history.normal_first_parent_path(root.id(), unrelated.id()),
        Err(HistoryError::InvalidNormalReplayRange {
            source_commit,
            target_commit,
        }) if source_commit == root.id() && target_commit == unrelated.id()
    ));
}

#[test]
fn rejects_merge_ancestry_from_a_normal_first_parent_range() {
    let context_id = ContextId::new();
    let main = BranchName::default();
    let feature = BranchName::new("feature").expect("branch");
    let root = commit(260, context_id, main.clone(), Vec::new());
    let left = commit(261, context_id, main.clone(), vec![root.id()]);
    let right = commit(262, context_id, feature.clone(), vec![root.id()]);
    let merge = commit(263, context_id, main.clone(), vec![left.id(), right.id()]);
    let history = CommitHistory::try_from_parts(
        context_id,
        [root.clone(), left, right, merge.clone()],
        [BranchHead::new(main, Some(merge.id()))],
    )
    .expect("valid merge graph");

    assert!(matches!(
        history.normal_first_parent_path(root.id(), merge.id()),
        Err(HistoryError::NonLinearAncestry { commit_id }) if commit_id == merge.id()
    ));
}

#[test]
fn delegates_merge_ancestry_to_a_deterministic_three_way_plan() {
    let context_id = ContextId::new();
    let main = BranchName::default();
    let feature = BranchName::new("feature").expect("branch");
    let root = commit(220, context_id, main.clone(), Vec::new());
    let left = commit(221, context_id, main.clone(), vec![root.id()]);
    let right = commit(222, context_id, feature.clone(), vec![root.id()]);

    let history = CommitHistory::try_from_parts(
        context_id,
        [right.clone(), root.clone(), left.clone()],
        [
            BranchHead::new(feature.clone(), Some(right.id())),
            BranchHead::new(main.clone(), Some(left.id())),
        ],
    )
    .expect("valid history");

    assert_eq!(
        history.merge_plan(&main, &feature).expect("three-way plan"),
        contextlab_versioning::MergePlan::ThreeWay {
            left: left.id(),
            right: right.id(),
            base: root.id(),
        }
    );
}

#[test]
fn preserves_fail_closed_errors_for_branch_and_head_lookup() {
    let context_id = ContextId::new();
    let main = BranchName::default();
    let unborn = BranchName::new("unborn").expect("branch");
    let absent = BranchName::new("absent").expect("branch");
    let root = commit(230, context_id, main.clone(), Vec::new());
    let unknown_head = CommitId::from_uuid(Uuid::from_u128(231));

    let unknown_head_error = CommitHistory::try_from_parts(
        context_id,
        [root.clone()],
        [BranchHead::new(main.clone(), Some(unknown_head))],
    )
    .expect_err("a head must point at a supplied commit");
    assert!(matches!(
        unknown_head_error,
        HistoryError::UnknownHead { branch, commit_id }
            if branch == main.to_string() && commit_id == unknown_head.to_string()
    ));

    let history = CommitHistory::try_from_parts(
        context_id,
        [root],
        [
            BranchHead::new(
                main.clone(),
                Some(CommitId::from_uuid(Uuid::from_u128(230))),
            ),
            BranchHead::new(unborn.clone(), None),
        ],
    )
    .expect("valid history");

    assert!(matches!(
        history.commits_for_branch(&absent),
        Err(HistoryError::UnknownBranch { branch }) if branch == absent.to_string()
    ));
    assert!(matches!(
        history.merge_plan(&absent, &main),
        Err(HistoryError::UnknownBranch { branch }) if branch == absent.to_string()
    ));
    assert!(matches!(
        history.merge_plan(&unborn, &main),
        Err(HistoryError::UnbornBranch { branch }) if branch == unborn.to_string()
    ));
}

#[test]
fn rejects_duplicate_branch_heads() {
    let context_id = ContextId::new();
    let branch = BranchName::default();

    let error = CommitHistory::try_from_parts(
        context_id,
        [],
        [
            BranchHead::new(branch.clone(), None),
            BranchHead::new(branch.clone(), None),
        ],
    )
    .expect_err("branch heads must be unique");

    assert!(matches!(
        error,
        HistoryError::DuplicateBranch { branch: actual } if actual == branch.to_string()
    ));
}
