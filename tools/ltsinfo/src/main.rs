use std::error::Error;
use std::fs::File;
use std::io::stdout;
use std::io::BufWriter;
use std::process::ExitCode;

use clap::Parser;
use clap::ValueEnum;

use mcrl2rust_io::io_aut::read_aut;
use mcrl2rust_io::io_aut::write_aut;
use mcrl2rust_reduction::branching_bisim_sigref;
use mcrl2rust_reduction::branching_bisim_sigref_naive;
use mcrl2rust_reduction::quotient_lts;
use mcrl2rust_reduction::strong_bisim_sigref;
use mcrl2rust_reduction::strong_bisim_sigref_naive;
use mcrl2rust_reduction::IndexedPartition;
use mcrl2rust_utilities::Timing;

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
    StrongBisimNaive,
    BranchingBisim,
    BranchingBisimNaive,
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

    let mut timing = Timing::new();
    let lts = read_aut(&file, cli.tau.unwrap_or_default())?;

    let partition: IndexedPartition = match cli.equivalence {
        Equivalence::StrongBisim => strong_bisim_sigref(&lts, &mut timing),
        Equivalence::StrongBisimNaive => strong_bisim_sigref_naive(&lts, &mut timing),
        Equivalence::BranchingBisim => branching_bisim_sigref(&lts, &mut timing),
        Equivalence::BranchingBisimNaive => branching_bisim_sigref_naive(&lts, &mut timing),
    };

    let mut quotient_time = timing.start("quotient");
    let quotient_lts = quotient_lts(
        &lts,
        &partition,
        matches!(cli.equivalence, Equivalence::BranchingBisim)
            || matches!(cli.equivalence, Equivalence::BranchingBisimNaive),
    );
    if let Some(file) = cli.output {
        let mut writer = BufWriter::new(File::create(file)?);
        write_aut(&mut writer, &quotient_lts)?;
    } else {
        write_aut(&mut stdout(), &quotient_lts)?;
    }
    quotient_time.finish();

    if cli.time {
        timing.print();
    }

    #[cfg(feature = "measure-allocs")]
    eprintln!("allocations: {}", MEASURE_ALLOC.number_of_allocations());

    Ok(ExitCode::SUCCESS)
}
