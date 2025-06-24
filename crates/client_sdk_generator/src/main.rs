use std::path::PathBuf;

use client_sdk_schema::{RESOURCE_INDEX, Schema};
use client_sdk_ts_codegen::{
    entrypoint::generate_entrypoint_module, generate_resource_module, loader::generate_loader,
    method::generate_method_modules,
};
use clap::{Parser as ClapParser, Subcommand, ValueEnum};

#[derive(ClapParser, Debug)]
#[clap(name = "client-sdk-generator")]
struct Args {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long, global = true, default_value = "portone-client-sdk.yml")]
    schema: PathBuf,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[clap(name = "generate")]
    Generate {
        out_dir: PathBuf,
        #[arg(long, value_enum)]
        generator: Generator,
    },
}

#[derive(ValueEnum, Clone, Copy, Debug)]
enum Generator {
    #[clap(name = "typescript")]
    TypeScript,
    #[clap(name = "dart")]
    Dart,
}

fn load_schema(path: &PathBuf) -> Schema {
    let schema = std::fs::read_to_string(path).unwrap();
    serde_yaml_ng::from_str(&schema).unwrap()
}

fn main() {
    let args = Args::parse();
    match args.command {
        Commands::Generate { out_dir, generator } => {
            println!("Generating code to {:?}", out_dir);
            match generator {
                Generator::TypeScript => {
                    println!("Generating TypeScript code");
                    let schema: Schema = load_schema(&args.schema);
                    let resource_index = schema.build_resource_index();
                    RESOURCE_INDEX.set(&resource_index, || {
                        generate_resource_module(&out_dir, "", &schema.resources, &out_dir);
                        generate_method_modules(&out_dir, &schema.methods);
                        generate_loader(&out_dir, &schema.methods);
                        generate_entrypoint_module(&out_dir, &schema.methods);
                    });
                }
                Generator::Dart => {
                    println!("Generating Dart code");
                    let schema: Schema = load_schema(&args.schema);
                    let resource_index = schema.build_resource_index();
                    RESOURCE_INDEX.set(&resource_index, || {
                        client_sdk_dart_codegen::generate_resources_module(
                            &schema.resources,
                            &out_dir,
                            "package:portone_flutter_sdk",
                        );
                    });
                    let mut child = std::process::Command::new("dart")
                        .arg("format")
                        .arg(&out_dir)
                        .spawn()
                        .unwrap();
                    child.wait().unwrap();
                }
            }
        }
    }
}
