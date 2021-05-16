use structopt::StructOpt;
mod git;
use std::path::PathBuf;

#[derive(Debug, StructOpt)]
#[structopt(name = "gl-mr", about = "Usage of gl-mr")]
struct Opt {
    /// Activate dry mode
    #[structopt(long)]
    dry: bool,

    /// Path to the git repository. Defaults to the current working directory
    #[structopt(parse(from_os_str), short, long, default_value = ".")]
    path: PathBuf,

    /// Git executable. Defaults to the executable that's in $PATH
    #[structopt(short, long, default_value = "git")]
    git: String,

    /// Enable dependent commit mode. Useful, when you want to have several MRs that depend on each other
    #[structopt(short, long)]
    dependent: bool,
}

fn main() {
    let opt = Opt::from_args();
    if !opt.path.exists() || !opt.path.is_dir() {
        eprintln!("Invalid path: {:?}", opt.path);
        std::process::exit(1);
    }
    let git = git::Git::new(None, opt.path, opt.dry);
    git::create_separate_merge_requests(&git, opt.dependent);
}
