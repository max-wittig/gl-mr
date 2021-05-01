use structopt::StructOpt;
mod git;
use std::path::PathBuf;

#[derive(Debug, StructOpt)]
#[structopt(name = "gl-mr", about = "Usage of gl-mr")]
struct Opt {
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,

    /// Activate dry mode
    #[structopt(long)]
    dry: bool,
}

fn main() {
    let opt = Opt::from_args();
    let git = git::Git::new(None, Some(PathBuf::from(".")), opt.dry);
    git::create_separate_merge_requests(&git);
}
