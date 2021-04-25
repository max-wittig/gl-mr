# gl-mr

Creates Merge-Requests on GitLab based on the commits in a separate branch.

This is quite handy, if you work with feature branches where your goal is to create separate
merge requests that depend on each other.

## Usage

```sh
USAGE:
    gl-mr [FLAGS] [OPTIONS]

FLAGS:
    -d, --debug      Activate debug mode
        --dry        Activate dry mode
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -g, --git <git>       [default: git]
    -p, --path <path>     [default: .]
```
