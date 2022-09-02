use git2::{Cred, PushOptions, RemoteCallbacks};
use indicatif::{ProgressBar, ProgressStyle};
use rdm_macros::{FromError, ToDoc};

use crate::config::Config;

#[derive(Debug, FromError, ToDoc)]
#[doc_prefix = "Push error:"]
#[doc_after_prefix = "pretty::RcDoc::line()"]
pub(crate) enum PushError {
    #[doc_to_string]
    GitError(git2::Error),
    #[doc_text = "HEAD is not on a branch"]
    HeadNotBranch,
}

pub(super) fn run(config: Config) -> Result<(), PushError> {
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

        cbs.push_transfer_progress(move |current, total, bytes| {
            if current == 0 {
                let (_, cols) = console::Term::stdout().size();
                pb = ProgressBar::new(total as u64)
                    .with_message("Sending objects")
                    .with_style(style.clone())
                    .with_tab_width(cols as usize);
                pb.tick();
            } else if current == total {
                pb.finish_with_message(format!("Sent {} bytes", bytes));
            } else {
                pb.set_position(current as u64);
            }
        });

        cbs.push_update_reference(|refname, status| match status {
            Some(msg) => Err(git2::Error::from_str(
                format!(
                    "Failed to update reference {} with message: {}",
                    refname, msg
                )
                .as_str(),
            )),
            None => Ok(()),
        });

        let mut remote = repo.find_remote(&remote_name)?;

        let mut push_opts = PushOptions::default();

        push_opts.remote_callbacks(cbs);

        log::info!(
            "Pushing {} to {}/{}",
            branch_name,
            remote_name,
            branch_name
        );

        remote.push(&[refspec], Some(&mut push_opts))?;

        log::info!("Successfully pushed your configuration to {}", remote_name);

        Ok(())
    } else {
        Err(PushError::HeadNotBranch)
    }
}
