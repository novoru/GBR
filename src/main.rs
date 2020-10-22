mod core;
mod gui;

use gui::window::run;

use std::path::Path;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long)]
    pub rom: String,
}


fn main() {
    let opt = Opt::from_args();
    let path = Path::new(&opt.rom);

    run(path);
}
