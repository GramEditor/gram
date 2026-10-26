use std::sync::Arc;

use anyhow::{Context as _, Result};
use buffer_diff::BufferDiff;
use collections::HashSet;
use futures::StreamExt;
use git::{
    repository::RepoPath,
    status::{DiffTreeType, FileStatus, StatusCode, TrackedStatus, TreeDiff, TreeDiffStatus},
};
use gpui::{
    App, AppContext, AsyncApp, AsyncWindowContext, Context, Entity, EventEmitter, SharedString, Subscription, Task,
    WeakEntity, Window,
};

use language::{Buffer, DiskState};
use text::BufferId;
use util::ResultExt;
use worktree::{FileType, Worktree};

use crate::{
    Project,
    git_store::{GitStoreEvent, Repository, RepositoryEvent, RepositoryState},
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DiffBase {
    Head,
    Merge { base_ref: SharedString },
}

impl DiffBase {
    pub fn is_merge_base(&self) -> bool {
        matches!(self, DiffBase::Merge { .. })
    }
}

pub struct BranchDiff {
    diff_base: DiffBase,
    repo: Option<Entity<Repository>>,
    project: Entity<Project>,
    base_commit: Option<SharedString>,
    head_commit: Option<SharedString>,
    tree_diff: Option<TreeDiff>,
    _subscription: Subscription,
    update_needed: postage::watch::Sender<()>,
    _task: Task<()>,
}

pub enum BranchDiffEvent {
    FileListChanged,
}

impl EventEmitter<BranchDiffEvent> for BranchDiff {}

impl BranchDiff {
    pub fn new(source: DiffBase, project: Entity<Project>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let git_store = project.read(cx).git_store().clone();
        let git_store_subscription = cx.subscribe_in(
            &git_store,
            window,
            move |this, _git_store, event, _window, cx| match event {
                GitStoreEvent::ActiveRepositoryChanged(_)
                | GitStoreEvent::RepositoryUpdated(
                    _,
                    RepositoryEvent::StatusesChanged
                    | RepositoryEvent::BranchChanged
                    | RepositoryEvent::MergeHeadsChanged,
                    true,
                )
                | GitStoreEvent::ConflictsUpdated => {
                    cx.emit(BranchDiffEvent::FileListChanged);
                    *this.update_needed.borrow_mut() = ();
                }
                _ => {}
            },
        );

        let (send, recv) = postage::watch::channel::<()>();
        let worker = window.spawn(cx, {
            let this = cx.weak_entity();
            async |cx| Self::handle_status_updates(this, recv, cx).await
        });
        let repo = git_store.read(cx).active_repository();

        Self {
            diff_base: source,
            repo,
            project,
            tree_diff: None,
            base_commit: None,
            head_commit: None,
            _subscription: git_store_subscription,
            _task: worker,
            update_needed: send,
        }
    }

    pub fn diff_base(&self) -> &DiffBase {
        &self.diff_base
    }

    pub async fn handle_status_updates(
        this: WeakEntity<Self>,
        mut recv: postage::watch::Receiver<()>,
        cx: &mut AsyncWindowContext,
    ) {
        Self::reload_tree_diff(this.clone(), cx).await.log_err();
        while recv.next().await.is_some() {
            let Ok(needs_update) = this.update(cx, |this, cx| {
                let mut needs_update = false;
                let active_repo = this.project.read(cx).git_store().read(cx).active_repository();
                if active_repo != this.repo {
                    needs_update = true;
                    this.repo = active_repo;
                } else if let Some(repo) = this.repo.as_ref() {
                    repo.update(cx, |repo, _| {
                        if let Some(branch) = &repo.branch
                            && let DiffBase::Merge { base_ref } = &this.diff_base
                            && let Some(commit) = branch.most_recent_commit.as_ref()
                            && &branch.ref_name == base_ref
                            && this.base_commit.as_ref() != Some(&commit.sha)
                        {
                            this.base_commit = Some(commit.sha.clone());
                            needs_update = true;
                        }

                        if repo.head_commit.as_ref().map(|c| &c.sha) != this.head_commit.as_ref() {
                            this.head_commit = repo.head_commit.as_ref().map(|c| c.sha.clone());
                            needs_update = true;
                        }
                    })
                }
                needs_update
            }) else {
                return;
            };

            if needs_update {
                Self::reload_tree_diff(this.clone(), cx).await.log_err();
            }
        }
    }

    pub fn status_for_buffer_id(&self, buffer_id: BufferId, cx: &App) -> Option<FileStatus> {
        let (repo, path) = self
            .project
            .read(cx)
            .git_store()
            .read(cx)
            .repository_and_path_for_buffer_id(buffer_id, cx)?;
        if self.repo() == Some(&repo) {
            return self.merge_statuses(
                repo.read(cx).status_for_path(&path).map(|status| status.status),
                self.tree_diff.as_ref().and_then(|diff| diff.entries.get(&path)),
            );
        }
        None
    }

    pub fn merge_statuses(
        &self,
        diff_from_head: Option<FileStatus>,
        diff_from_merge_base: Option<&TreeDiffStatus>,
    ) -> Option<FileStatus> {
        match (diff_from_head, diff_from_merge_base) {
            (None, None) => None,
            (Some(diff_from_head), None) => Some(diff_from_head),
            (Some(diff_from_head @ FileStatus::Unmerged(_)), _) => Some(diff_from_head),

            // file does not exist in HEAD
            // but *does* exist in work-tree
            // and *does* exist in merge-base
            (
                Some(FileStatus::Untracked)
                | Some(FileStatus::Tracked(TrackedStatus {
                    index_status: StatusCode::Added,
                    worktree_status: _,
                })),
                Some(_),
            ) => Some(FileStatus::Tracked(TrackedStatus {
                index_status: StatusCode::Modified,
                worktree_status: StatusCode::Modified,
            })),

            // file exists in HEAD
            // but *does not* exist in work-tree
            (Some(diff_from_head), Some(diff_from_merge_base)) if diff_from_head.is_deleted() => {
                match diff_from_merge_base {
                    TreeDiffStatus::Added => None, // unchanged, didn't exist in merge base or worktree
                    _ => Some(diff_from_head),
                }
            }

            // file exists in HEAD
            // and *does* exist in work-tree
            (Some(FileStatus::Tracked(_)), Some(tree_status)) => Some(FileStatus::Tracked(TrackedStatus {
                index_status: match tree_status {
                    TreeDiffStatus::Added => StatusCode::Added,
                    _ => StatusCode::Modified,
                },
                worktree_status: match tree_status {
                    TreeDiffStatus::Added => StatusCode::Added,
                    _ => StatusCode::Modified,
                },
            })),

            (_, Some(diff_from_merge_base)) => Some(diff_status_to_file_status(diff_from_merge_base)),
        }
    }

    pub async fn reload_tree_diff(this: WeakEntity<Self>, cx: &mut AsyncWindowContext) -> Result<()> {
        let task = this.update(cx, |this, cx| {
            let DiffBase::Merge { base_ref } = this.diff_base.clone() else {
                return None;
            };
            let Some(repo) = this.repo.as_ref() else {
                this.tree_diff.take();
                return None;
            };
            repo.update(cx, |repo, cx| {
                Some(repo.diff_tree(
                    DiffTreeType::MergeBase {
                        base: base_ref,
                        head: "HEAD".into(),
                    },
                    cx,
                ))
            })
        })?;
        let Some(task) = task else { return Ok(()) };

        let diff = task.await??;
        this.update(cx, |this, cx| {
            this.tree_diff = Some(diff);
            cx.emit(BranchDiffEvent::FileListChanged);
            cx.notify();
        })
    }

    pub fn repo(&self) -> Option<&Entity<Repository>> {
        self.repo.as_ref()
    }

    pub fn load_buffers(&mut self, cx: &mut Context<Self>) -> Vec<DiffBuffer> {
        let mut output = Vec::default();
        let Some(repo) = self.repo.clone() else {
            return output;
        };

        self.project.update(cx, |_project, cx| {
            let mut seen = HashSet::default();

            for item in repo.read(cx).cached_status() {
                seen.insert(item.repo_path.clone());
                let branch_diff = self
                    .tree_diff
                    .as_ref()
                    .and_then(|t| t.entries.get(&item.repo_path))
                    .cloned();
                let Some(status) = self.merge_statuses(Some(item.status), branch_diff.as_ref()) else {
                    continue;
                };
                if !status.has_changes() {
                    continue;
                }

                let Some(project_path) = repo.read(cx).repo_path_to_project_path(&item.repo_path, cx) else {
                    continue;
                };
                let task = Self::load_buffer(branch_diff, project_path, item.status, repo.clone(), cx);

                output.push(DiffBuffer {
                    repo_path: item.repo_path.clone(),
                    load: task,
                    file_status: item.status,
                });
            }
            let Some(tree_diff) = self.tree_diff.as_ref() else {
                return;
            };

            for (path, branch_diff) in tree_diff.entries.iter() {
                if seen.contains(path) {
                    continue;
                }

                let Some(project_path) = repo.read(cx).repo_path_to_project_path(path, cx) else {
                    continue;
                };
                let file_status = diff_status_to_file_status(branch_diff);
                let task = Self::load_buffer(Some(branch_diff.clone()), project_path, file_status, repo.clone(), cx);

                output.push(DiffBuffer {
                    repo_path: path.clone(),
                    load: task,
                    file_status,
                });
            }
        });
        output
    }

    fn load_buffer(
        branch_diff: Option<git::status::TreeDiffStatus>,
        project_path: crate::ProjectPath,
        file_status: FileStatus,
        repo: Entity<Repository>,
        cx: &Context<'_, Project>,
    ) -> Task<Result<(Entity<Buffer>, Entity<BufferDiff>)>> {
        log::info!("TEMP: BranchDiff::load_buffer");
        cx.spawn(async move |project, cx| {
            let buffer = match project
                .update(cx, |project, cx| project.open_buffer(project_path.clone(), cx))?
                .await
            {
                Ok(buffer) => buffer,
                Err(_) => {
                    return Self::open_representation(project_path, file_status, project, repo, cx).await;
                }
            };

            let changes = if let Some(entry) = branch_diff {
                let oid = match entry {
                    git::status::TreeDiffStatus::Added => None,
                    git::status::TreeDiffStatus::Modified { old, .. }
                    | git::status::TreeDiffStatus::Deleted { old } => Some(old),
                };
                project
                    .update(cx, |project, cx| {
                        project.git_store().update(cx, |git_store, cx| {
                            git_store.open_diff_since(oid, buffer.clone(), repo, cx)
                        })
                    })?
                    .await?
            } else {
                project
                    .update(cx, |project, cx| project.open_uncommitted_diff(buffer.clone(), cx))?
                    .await?
            };
            Ok((buffer, changes))
        })
    }

    /// Create a representation of the contents of a non-text file.
    ///
    /// This will show the revision of a submodule or the size of
    /// a binary file.
    async fn open_representation(
        project_path: crate::ProjectPath,
        file_status: FileStatus,
        project: WeakEntity<Project>,
        repo: Entity<Repository>,
        cx: &mut AsyncApp,
    ) -> Result<(Entity<Buffer>, Entity<BufferDiff>)> {
        let worktree = project
            .update(cx, |project, cx| {
                project
                    .worktree_store()
                    .read(cx)
                    .worktree_for_id(project_path.worktree_id, cx)
            })?
            .context("found no worktree for project_path worktree_id")?;

        let (path, metadata) = worktree
            .update(cx, |worktree, cx| match worktree {
                Worktree::Local(this) => {
                    let path = this.absolutize(&project_path.path);
                    let fs = this.fs().clone();
                    Ok(cx.background_spawn(async move {
                        let metadata = fs.metadata(&path).await?;
                        Ok::<_, anyhow::Error>((path, metadata))
                    }))
                }
                Worktree::Remote(_) => anyhow::bail!("remote worktrees can't yet load files"),
            })??
            .await?;
        let metadata = metadata.with_context(|| format!("found no metadata for {}", path.display()))?;

        let repository_state = repo.update(cx, |repo, _| repo.repository_state.clone())?.await.unwrap();
        let repository = match repository_state {
            RepositoryState::Local(local) => local,
            RepositoryState::Remote(_) => {
                anyhow::bail!("remote repositories can't yet show submodule diffs");
            }
        };

        let file_is_new = file_status.is_untracked() || file_status.is_created();

        let (old, new, file_type) = if metadata.is_dir {
            let submodule = repository.backend.submodule(project_path.path.clone()).await?;

            let new = if submodule.is_dirty {
                format!("Subproject commit {}+dirty", submodule.oid)
            } else {
                format!("Subproject commit {}", submodule.oid)
            };

            let old = if file_is_new {
                String::new()
            } else {
                let oid = repository.backend.submodule_head(project_path.path.clone()).await?;
                format!("Subproject commit {}", oid)
            };
            (old, new, FileType::Submodule)
        } else {
            let new_size = metadata.len;
            let new = format!("{} byte binary file", new_size);
            let old = if file_is_new {
                String::new()
            } else {
                let old_size = repository.backend.object_head_size(project_path.path.clone()).await?;
                format!("{} byte binary file", old_size)
            };
            (old, new, FileType::Binary)
        };

        let buffer = cx.new(|cx| {
            let text_buffer = text::Buffer::new(text::ReplicaId::LOCAL, cx.entity_id().as_non_zero_u64().into(), new);

            let path = project_path.path;
            let file = Arc::new(worktree::File {
                entry_id: None,
                worktree,
                path,
                disk_state: DiskState::Present {
                    mtime: metadata.mtime,
                    size: metadata.len,
                },
                file_type,
                is_local: true,
                is_private: false,
            });
            Buffer::build_representation(text_buffer, file)
        })?;

        let diff = cx.new(|cx| {
            let snapshot = buffer.read(cx).snapshot();
            BufferDiff::new(&snapshot.text, cx)
        })?;
        let _ = diff
            .update(cx, |diff, cx| {
                let snapshot = buffer.read(cx).snapshot();
                diff.set_base_text(Some(Arc::from(old)), None, snapshot.text, cx)
            })?
            .await;

        Ok((buffer, diff))
    }
}

fn diff_status_to_file_status(branch_diff: &git::status::TreeDiffStatus) -> FileStatus {
    match branch_diff {
        git::status::TreeDiffStatus::Added => FileStatus::Tracked(TrackedStatus {
            index_status: StatusCode::Added,
            worktree_status: StatusCode::Added,
        }),
        git::status::TreeDiffStatus::Modified { .. } => FileStatus::Tracked(TrackedStatus {
            index_status: StatusCode::Modified,
            worktree_status: StatusCode::Modified,
        }),
        git::status::TreeDiffStatus::Deleted { .. } => FileStatus::Tracked(TrackedStatus {
            index_status: StatusCode::Deleted,
            worktree_status: StatusCode::Deleted,
        }),
    }
}

#[derive(Debug)]
pub struct DiffBuffer {
    pub repo_path: RepoPath,
    pub file_status: FileStatus,
    pub load: Task<Result<(Entity<Buffer>, Entity<BufferDiff>)>>,
}
