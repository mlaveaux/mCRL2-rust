
use std::fs::File;

use anyhow::Result;

use clap::Parser;

use io::aut::{read_aut, LTS};
use leptos::{component, create_signal, view, IntoView, SignalGet, SignalSet};

#[derive(Parser, Debug)]
#[command(
    name = "Maurice Laveaux",
    about = "A lts viewing tool",
)]
pub struct Cli {
    #[arg(value_name = "FILE")]
    labelled_transition_system: String,

}

#[component]
fn App() -> impl IntoView {
    view! {
        <svg viewBox="-400,400,-400,400" width="928" height="400" style="max-width: 100% height: auto">
            <line x1="-13.694089066909036" y1="5.175125614131555" x2="15.510306910179446" y2="-2.1195359421896867"></line>
            <circle>Test</circle>
        </svg>
    }
}

fn main() -> Result<()>
{
    env_logger::init();
    console_error_panic_hook::set_once();

    let cli = Cli::parse();

    let file = File::open(cli.labelled_transition_system)?;
    let lts = read_aut(file).unwrap();

    // Show the main application.
    leptos::mount_to_body(|| view! { <App /> });

    Ok(())
}
