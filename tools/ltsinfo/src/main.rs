use std::fs::File;

use anyhow::Result as AnyResult;
use clap::Parser;
use io::io_aut::read_aut;
use lts::strong_bisim_sigref;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[derive(clap::Parser, Debug)]
#[command(name = "Maurice Laveaux", about = "A command line rewriting tool")]
struct Cli {
    filename: String,
}

fn main() -> AnyResult<()> {
    env_logger::init();

    let cli = Cli::parse();


    let file = File::open(cli.filename)?;
    let lts = read_aut(&file).unwrap();

    strong_bisim_sigref(&lts);

    Ok(())
}