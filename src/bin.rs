use clap::{Parser, Subcommand};
use roblox_install;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug, Clone, Copy)]
enum Command {
    ApplicationPath,
    BuiltInPluginsPath,
    ContentPath,
    PluginsPath,
}

fn main() {
    let args = Args::parse();

    let roblox_studio = match roblox_install::RobloxStudio::locate() {
        Ok(roblox_studio) => roblox_studio,
        Err(error) => {
            eprintln!("Couldn't locate Roblox Studio: {error}");
            std::process::exit(1);
        }
    };

    match args.command {
        Command::ApplicationPath => {
            print!("{}", roblox_studio.application_path().display())
        }
        Command::BuiltInPluginsPath => {
            println!("{}", roblox_studio.built_in_plugins_path().display())
        }
        Command::ContentPath => {
            println!("{}", roblox_studio.content_path().display())
        }
        Command::PluginsPath => {
            println!("{}", roblox_studio.plugins_path().display())
        }
    }
}
