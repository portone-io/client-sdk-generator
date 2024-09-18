use std::path::PathBuf;

use browser_sdk_schema::Schema;
use browser_sdk_ts_codegen::{
    entrypoint::generate_entrypoint_module, generate_resource_module, loader::generate_loader,
    method::generate_method_modules,
};
use clap::{Parser as ClapParser, Subcommand, ValueEnum};

#[derive(ClapParser, Debug)]
#[clap(name = "generator-cli")]
struct Args {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long, global = true, default_value = "browser-sdk.yaml")]
    schema: PathBuf,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[clap(name = "generate")]
    Generate {
        out_dir: PathBuf,
        #[arg(long, value_enum, default_value_t = Generator::TypeScript)]
        generator: Generator,
    },
}

#[derive(ValueEnum, Clone, Copy, Debug)]
enum Generator {
    #[clap(name = "typescript")]
    TypeScript,
}

fn load_schema(path: &PathBuf) -> Schema {
    let schema = std::fs::read_to_string(path).unwrap();
    serde_yaml::from_str(&schema).unwrap()
}

fn main() {
    let args = Args::parse();
    println!("{:?}", args);

    match args.command {
        Commands::Generate { out_dir, generator } => {
            println!("Generating code to {:?}", out_dir);
            match generator {
                Generator::TypeScript => {
                    println!("Generating TypeScript code");
                    let schema: Schema = load_schema(&args.schema);
                    generate_resource_module(&out_dir, "", &schema.resources, &out_dir);
                    generate_method_modules(&out_dir, &schema.methods);
                    generate_loader(&out_dir, &schema.methods);
                    generate_entrypoint_module(&out_dir, &schema.methods);
                }
            }
        }
    }
}
