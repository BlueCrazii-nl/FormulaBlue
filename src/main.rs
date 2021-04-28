use colored::Colorize;

mod config;
mod apis;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    println!("Starting FormulaBlue v{}", VERSION);
    print!("Reading configuration...");

    let cfg = config::read();
    let cfg_verify = cfg.verify();
    if !cfg_verify.0 {
        print!("{}\n", "FAIL".red());
        println!("Configuration did not pass checks: Field '{}' is empty.", cfg_verify.1);
        std::process::exit(1);
    } else {
        print!("{}\n", "OK".green());
    }

    print!("Attempting to log in to F1TV...");
    let login_response = apis::f1tv::login::do_login(&cfg.f1_username.unwrap(), &cfg.f1_password.unwrap());
    if login_response.is_err() {
        print!("{}\n", "FAIL".red());

        let err = login_response.err().unwrap();
        if err.detail.is_some() {
            println!("Login to F1TV failed. The reason is known as follows: '{}'", err.detail.unwrap());
        } else {
            println!("Login to F1TV failed. The reason is unknown. (status: {})", err.status.unwrap());
        }

        std::process::exit(1);
    } else {
        print!("{}\n", "OK".green());
    }


}
