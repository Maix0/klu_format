//use klu;
use std::path::PathBuf;
use structopt::{self, StructOpt};

mod cmds;

#[derive(Debug, StructOpt)]
#[structopt(about = "Klu archive tool")]
struct Commands {
    #[structopt(subcommand)]
    sub: SubCmd,
}

#[derive(Debug, StructOpt)]
enum SubCmd {
    #[structopt(about = "Extract archive")]
    Extract {
        #[structopt(parse(from_os_str))]
        #[structopt(about = "Path to archive")]
        archive: PathBuf,
        #[structopt(parse(from_os_str))]
        #[structopt(about = "Path where archive's file will be released")]
        path: PathBuf,
    },
    #[structopt(about = "Pack files to archive")]
    Pack {
        #[structopt(parse(from_os_str))]
        #[structopt(about = "Path to archive")]
        archive: PathBuf,
        #[structopt(parse(from_os_str))]
        #[structopt(about = "Path where archive will be created")]
        path: PathBuf,
    },
    #[structopt(about = "List archive's files")]
    List {
        #[structopt(parse(from_os_str))]
        #[structopt(about = "Path to archive")]
        archive: PathBuf,
    },
}

fn main() {
    let opt = Commands::from_args();
    match match opt.sub {
        SubCmd::Extract { path, archive } => cmds::extract(path, archive),
        SubCmd::Pack { path, archive } => cmds::pack(path, archive),
        SubCmd::List { archive } => cmds::list(archive),
    } {
        Ok(_) => (),
        Err(e) => {
            structopt::clap::Error::with_description(&e, structopt::clap::ErrorKind::InvalidValue)
                .exit();
        }
    }
}
