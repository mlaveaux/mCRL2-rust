use std::error::Error;

pub use duct::cmd;

#[cfg(not(unix))]
#[allow(clippy::ptr_arg)] // For other configurations the mutability is relevant.
fn add_target_flag(_arguments: &mut Vec<String>) {}

#[cfg(target_os = "linux")]
fn add_target_flag(arguments: &mut Vec<String>) {
    arguments.push("--target".to_string());
    arguments.push( "x86_64-unknown-linux-gnu".to_string());
}


#[cfg(target_os = "macos")]
fn add_target_flag(arguments: &mut Vec<String>) {
    arguments.push("--target".to_string());
    arguments.push( "x86_64-apple-darwin".to_string());
}

///
/// Run the tests with the address sanitizer enabled to detect memory issues in unsafe and C++ code.
///
/// This only works under Linux and MacOS currently and requires the nightly toolchain.
///
pub fn address_sanitizer(cargo_arguments: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut arguments: Vec<String> = vec![
        "test".to_string(),
        "-Zbuild-std".to_string(),
    ];

    add_target_flag(&mut arguments);
    arguments.extend(cargo_arguments);

    cmd("cargo", arguments)
        .env("CARGO_INCREMENTAL", "0")
        .env("RUSTFLAGS", "-Zsanitizer=address")
        .env("CFLAGS", "-fsanitize=address")
        .env("CXXFLAGS", "-fsanitize=address")
        .run()?;
    println!("ok.");

    Ok(())
}

///
/// Run the tests with the thread sanitizer enabled to detect data race conditions.
///
/// This only works under Linux and MacOS currently and requires the nightly toolchain.
///
pub fn thread_sanitizer(cargo_arguments: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut arguments: Vec<String> = vec![
        "test".to_string(),
        "-Zbuild-std".to_string(),
    ];

    add_target_flag(&mut arguments);
    arguments.extend(cargo_arguments);

    cmd("cargo", arguments)
        .env("CARGO_INCREMENTAL", "0")
        .env("RUSTFLAGS", "-Zsanitizer=thread")
        .env("CFLAGS", "-fsanitize=thread")
        .env("CXXFLAGS", "-fsanitize=thread")
        .run()?;
    println!("ok.");

    Ok(())
}