use pretty::RcDoc;
use rdm_macros::{FromError, ToDoc};

use crate::config::Config;

#[derive(Debug, FromError, ToDoc)]
#[doc_prefix = "list error"]
pub(crate) enum ListError {
    #[doc_to_string]
    GitError(git2::Error),
    #[doc_to_string]
    IoError(std::io::Error),
    #[doc_to_string]
    ToString(std::string::FromUtf8Error),
}

pub(super) fn run(config: Config) -> Result<(), ListError> {
    let repo = config.repo;

    let doc = RcDoc::<()>::text("Available remotes:")
        .append(RcDoc::line())
        .append(
            RcDoc::intersperse(
                repo.remotes()?.into_iter().map(|remote_name| {
                    let remote_name = remote_name.unwrap();
                    let remote = repo.find_remote(remote_name).unwrap();
                    let url = remote.url().unwrap();
                    RcDoc::text(format!("{}: {}", remote_name, url))
                }),
                RcDoc::line(),
            )
            .nest(1)
            .group(),
        );
    let mut buf = Vec::new();
    let (_, cols) = console::Term::stdout().size();
    doc.render(cols.into(), &mut buf)?;
    let str = String::from_utf8(buf)?;
    println!("{}", str);
    Ok(())
}
