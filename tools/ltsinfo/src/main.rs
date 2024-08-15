use std::{error::Error, fs::File, io::stdout, process::ExitCode};

use clap::{Parser, ValueEnum};
use io::io_aut::{read_aut, write_aut};
use lts::{branching_bisim_sigref, quotient_lts, strong_bisim_sigref, Partition};

#[cfg(feature = "measure-allocs")]
#[global_allocator]
static MEASURE_ALLOC: unsafety::AllocCounter = unsafety::AllocCounter;

#[cfg(feature = "measure-allocs")]
use log::info;

#[cfg(not(target_env = "msvc"))]
#[cfg(not(feature = "measure-allocs"))]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[derive(Clone, Debug, ValueEnum)]
enum Equivalence {
    StrongBisim,
    BranchingBisim,
}

#[derive(clap::Parser, Debug)]
#[command(name = "Maurice Laveaux", about = "A command line rewriting tool")]
struct Cli {
    equivalence: Equivalence,

    filename: String,
    
    output: Option<String>,
}

fn main() -> Result<ExitCode, Box<dyn Error>> {
    env_logger::init();

    let cli = Cli::parse();

    let file = File::open(cli.filename)?;
    let lts = read_aut(&file)?;

    let partition = match cli.equivalence {
        Equivalence::StrongBisim => {
            strong_bisim_sigref(&lts)
        },
        Equivalence::BranchingBisim => {
            branching_bisim_sigref(&lts)
        }
    };
    let quotient_lts = quotient_lts(&lts, &partition);

    if let Some(file) = cli.output {
        let mut writer = File::create(file)?;
        write_aut(&mut writer, &quotient_lts)?;
    } else {
        write_aut(&mut stdout(), &quotient_lts)?;
    }

    partition.block_number(0);

    #[cfg(feature = "measure-allocs")]
    info!("Allocations: {}", MEASURE_ALLOC.number_of_allocations());

    Ok(ExitCode::SUCCESS)
}