mod crypto;
mod database;
mod diff;
mod extraction;
mod ipc;
mod matching;
mod model;
mod normalization;
mod vault;

fn main() {
    eprintln!("Job Vault companion started");

    if let Err(e) = ipc::run() {
        eprintln!("Fatal error: {e}");
        std::process::exit(1);
    }
}
