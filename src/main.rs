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

    #[structopt(short, long)]
    path: String,
}

fn main() {
    let opt = Opt::from_args();
    let path = if opt.path.is_empty() {
        String::from(".")
    } else {
        opt.path
    };
    let git = git::Git::new(None, Some(PathBuf::from(path)), opt.dry);
    git::create_separate_merge_requests(&git);
}
