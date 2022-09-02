use git2::{
    build::CheckoutBuilder, Cred, FetchOptions, IndexConflict, MergeAnalysis,
    PushOptions, RemoteCallbacks,
};
use indicatif::{ProgressBar, ProgressStyle};
use rdm_macros::{FromError, ToDoc};

use crate::config::Config;

#[derive(Debug, FromError, ToDoc)]
#[doc_prefix = "Push error:"]
#[doc_after_prefix = "pretty::RcDoc::line()"]
pub(crate) enum PullError {
    #[doc_to_string]
    GitError(git2::Error),
    #[doc_text = "HEAD is not on a branch"]
    HeadNotBranch,
    #[doc_to_string]
    FromUtf8Error(std::string::FromUtf8Error),
}

pub(super) fn run(config: Config) -> Result<(), PullError> {
    let repo = config.repo;

    let head = repo.head()?;

    if head.is_branch() {
        let mut cbs = RemoteCallbacks::new();

        cbs.credentials(|_, username, _| {
            Cred::ssh_key_from_agent(username.unwrap())
        });

        let mut pb = ProgressBar::new(0);
        let style =
            ProgressStyle::with_template("[{msg}] {wide_bar} {pos}/{len}")
                .unwrap();

        let branch_name = head.shorthand().unwrap().to_string();
        let refspec = head.name().unwrap();

        let config = repo.config()?;

        let remote_key = format!("branch.{}.remote", branch_name);

        let remote_name = config.get_string(&remote_key)?;

        let mut remote = repo.find_remote(&remote_name)?;

        cbs.transfer_progress(move |stats| {
            if stats.received_objects() == 0 {
                pb = ProgressBar::new(stats.total_objects() as u64)
                    .with_style(style.clone())
                    .with_message("Receiving objects");
                pb.tick();
            } else if stats.received_objects() == stats.total_objects() {
                pb.finish_with_message(format!(
                    "Received {} bytes",
                    stats.received_bytes()
                ));

                pb = ProgressBar::new(stats.total_deltas() as u64)
                    .with_message("Resolving deltas")
                    .with_style(style.clone());
            } else if stats.received_objects() < stats.total_objects() {
                pb.set_position(stats.received_objects() as u64);
            } else if stats.indexed_deltas() > 0 {
                pb.set_position(stats.indexed_deltas() as u64);
            } else {
                pb.finish_with_message("Resolved deltas");
            }

            true
        });

        let mut fetch_opts = FetchOptions::default();

        fetch_opts.remote_callbacks(cbs);

        log::info!("Fetching {}/{}", remote_name, branch_name);

        remote.fetch(&[refspec], Some(&mut fetch_opts), None)?;

        let fetch_head = repo.find_reference("FETCH_HEAD")?;

        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;

        let analysis = repo.merge_analysis(&[&fetch_commit])?;

        match analysis.0 {
            MergeAnalysis::ANALYSIS_FASTFORWARD => {
                let ref_name = format!("refs/heads/{}", branch_name);
                match repo.find_reference(&ref_name) {
                    Ok(mut ref_) => {
                        let msg = format!(
                            "Setting {} to {}",
                            ref_name,
                            fetch_commit.id()
                        );
                        ref_.set_target(fetch_commit.id(), &msg)?;
                        repo.set_head(&ref_name)?;
                        repo.checkout_head(Some(
                            CheckoutBuilder::default().force(),
                        ))?;
                    }
                    Err(_) => {
                        repo.reference(
                            &ref_name,
                            fetch_commit.id(),
                            true,
                            format!(
                                "Setting {} to {}",
                                branch_name,
                                fetch_commit.id()
                            )
                            .as_str(),
                        )?;
                        repo.set_head(&ref_name)?;
                        repo.checkout_head(Some(
                            CheckoutBuilder::default()
                                .allow_conflicts(true)
                                .conflict_style_merge(true)
                                .force(),
                        ))?;
                    }
                }

                log::info!("Fast forwarded to FETCH_HEAD.")
            }
            MergeAnalysis::ANALYSIS_NORMAL => {
                let head_commit =
                    repo.reference_to_annotated_commit(&repo.head()?)?;
                let local_tree = repo.find_commit(head_commit.id())?.tree()?;
                let remote_tree =
                    repo.find_commit(fetch_commit.id())?.tree()?;

                let ancestor = repo
                    .find_commit(
                        repo.merge_base(head_commit.id(), fetch_commit.id())?,
                    )?
                    .tree()?;

                let mut idx = repo.merge_trees(
                    &ancestor,
                    &local_tree,
                    &remote_tree,
                    None,
                )?;

                if idx.has_conflicts() {
                    log::warn!("Merge conflicts detected...");
                    for conflict in idx.conflicts()? {
                        let conflict = conflict?;
                        println!(
                            "{:2}{}",
                            "",
                            String::from_utf8(conflict.our.unwrap().path)?
                        );
                    }

                    repo.checkout_index(Some(&mut idx), None)?;
                    return Ok(());
                }

                let result_tree = repo.find_tree(idx.write_tree_to(&repo)?)?;

                let msg = format!(
                    "Merge: {} into {}",
                    fetch_commit.id(),
                    head_commit.id()
                );
                let sig = repo.signature()?;
                let local_commit = repo.find_commit(head_commit.id())?;
                let remote_commit = repo.find_commit(fetch_commit.id())?;

                repo.commit(
                    Some("HEAD"),
                    &sig,
                    &sig,
                    &msg,
                    &result_tree,
                    &[&local_commit, &remote_commit],
                )?;

                repo.checkout_head(None)?;

                log::info!(
                    "Successfully merged FETCH_HEAD into {}",
                    branch_name
                );
            }
            _ => log::info!("Your configuration is already up to date."),
        };

        Ok(())
    } else {
        Err(PullError::HeadNotBranch)
    }
}
