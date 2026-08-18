mod database;
mod ipc;
mod vault;

fn main() {
    eprintln!("Job Vault companion started");

    if let Err(e) = ipc::run() {
        eprintln!("Fatal error: {e}");
        std::process::exit(1);
    }
}
