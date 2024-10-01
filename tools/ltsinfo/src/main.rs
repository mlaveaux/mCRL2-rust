use std::{error::Error, fs::File, io::{stdout, BufWriter}, process::ExitCode};

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

    #[arg(short, long)]
    tau: Option<Vec<String>>,

    #[arg(long)]
    time: bool,
}

fn main() -> Result<ExitCode, Box<dyn Error>> {
    env_logger::init();

    let cli = Cli::parse();

    let file = File::open(cli.filename)?;

    let lts = read_aut(&file, cli.tau.unwrap_or_default())?;

    let start = std::time::Instant::now();
    let partition = match cli.equivalence {
        Equivalence::StrongBisim => strong_bisim_sigref(&lts),
        Equivalence::BranchingBisim => branching_bisim_sigref(&lts),
    };

    if cli.time {
        eprintln!("reduction: {:.3}s", start.elapsed().as_secs_f64());
    }

    let quotient_lts = quotient_lts(&lts, &partition, matches!(cli.equivalence, Equivalence::BranchingBisim));
    if let Some(file) = cli.output {
        let mut writer = BufWriter::new(File::create(file)?);
        write_aut(&mut writer, &quotient_lts)?;
    } else {
        write_aut(&mut stdout(), &quotient_lts)?;
    }

    partition.block_number(0);

    #[cfg(feature = "measure-allocs")]
    eprintln!("allocations: {}", MEASURE_ALLOC.number_of_allocations());

    Ok(ExitCode::SUCCESS)
}
