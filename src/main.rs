use structopt::StructOpt;
mod credentials;
mod git;
use std::path::PathBuf;

#[derive(Debug, StructOpt)]
#[structopt(name = "gl-mr", about = "Usage of gl-mr")]
struct Opt {
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,
}

fn main() {
    let opt = Opt::from_args();
    let git = git::Git::new(
        None,
        Some(PathBuf::from("/Users/max/Documents/Development/code")),
    );
    git::create_separate_merge_requests(&git);
}
