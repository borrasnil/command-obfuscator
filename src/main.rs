use std::io::{self, Read};

use clap::{Parser, ValueEnum};
use command_obfuscator::modules::command::reverse::ReverseObfuscator;
use command_obfuscator::modules::string::hex::HexObfuscator;
use command_obfuscator::modules::string::param::ParamObfuscator;
use command_obfuscator::modules::string::quotes::QuotesObfuscator;
use command_obfuscator::{OS, Pipeline};

#[derive(Clone, ValueEnum, Debug)]
enum Module {
    Quotes,
    Hex,
    Param,
    Reverse,
}

#[derive(Parser)]
#[command(name = "boo", about = "Bash obfuscation tool")]
struct Args {
    /// Command to obfuscate (reads from stdin if omitted)
    #[arg(short, long)]
    command: Option<String>,

    /// Modules to apply in order (default: param,quotes)
    #[arg(short, long, value_enum, num_args = 1.., value_delimiter = ',')]
    module: Option<Vec<Module>>,
}

fn main() {
    let args = Args::parse();

    let command = match args.command {
        Some(c) => c,
        None => {
            let mut buf = String::new();
            io::stdin()
                .read_to_string(&mut buf)
                .expect("failed to read stdin");
            buf.trim_end_matches('\n').to_string()
        }
    };

    let modules: Vec<Module> = args
        .module
        .unwrap_or_else(|| vec![Module::Param, Module::Quotes]);

    let mut pipeline = Pipeline::new(OS::Linux);
    for m in modules {
        match m {
            Module::Quotes => {
                pipeline = pipeline.add(QuotesObfuscator);
            }
            Module::Hex => {
                pipeline = pipeline.add(HexObfuscator);
            }
            Module::Param => {
                pipeline = pipeline.add(ParamObfuscator);
            }
            Module::Reverse => {
                pipeline = pipeline.add(ReverseObfuscator);
            }
        }
    }

    println!("{}", pipeline.run(&command));
}
