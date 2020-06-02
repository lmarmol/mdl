#[macro_use]
extern crate clap;

mod lib;
use lib::{commands, settings::Settings};

#[tokio::main]
async fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = clap::App::from_yaml(yaml).get_matches();

    let mut settings = Settings::new();
    let _ = settings.load_from_file("mdl.toml");

    match matches.subcommand() {
        ("login", Some(login_matches)) => {
            let email = login_matches.value_of("user").unwrap();
            commands::login(&mut settings, email).await
        }
        ("list", Some(_)) => commands::list(&settings).await,
        ("download", Some(download_matches)) => {
            let groups = download_matches.values_of("id").unwrap().collect();
            commands::download(&settings, &groups).await
        }
        _ => Ok(()),
    }
    .expect("The command failed");
}
