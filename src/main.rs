mod installer;
mod init;
mod generate_launch_script;
mod launcher_profile;
mod lib_get_forgelike;
mod lib_get_fabric;

use clap::{Parser,Subcommand};

#[derive(Parser, Debug)]
pub struct Options {
    #[command(subcommand)]
    pub args: Argument,
}

#[derive(Subcommand, Debug)]
#[command(about = "generate launch script", long_about = "You need two args, the game versions/<version> directory and the client type of the game.example:wml generate --game_path versions/1.21.8/version.json --client_type vanilla --mod_json $HOME/.minecraft/versions/fabric-loader-0.17.2-1.21.8/fabric-loader-0.17.2-1.21.8.json --output_path $HOME/.minecraft/versions/fabric-loader-0.17.2-1.21.8/")]
pub enum Argument {
    Install {
        #[arg(short, long)]
        version: String,
        #[arg(long)]
        name: String,
    },
    Modpack {
        #[arg(short, long)]
        file: String,
        #[arg(long)]
        name: String,
    },
    List {
    },
    Generate {
        #[arg(short,long)]
        game_path:String,
        #[arg(short,long)]
        client:String,
        #[arg(short,long)]
        mod_json:Option<String>,
        #[arg(short,long)]
        output_path:String
    }
}

fn main () {
    init::main();
    installer::downloade_game().unwrap();
    generate_launch_script::generate_launch_sh();
}
