use anyhow::Context;
use clap::{Args, Parser, Subcommand};

use viv_script::{build, CompilerOptions};

#[derive(Subcommand)]
enum CompilerCommand {
    /// Compile and run file
    Run { input_file: String },
}

#[derive(Args)]
struct DebugArguments {
    /// Turn off compiler optimizations
    #[arg(short = 'd', long, global = true)]
    dont_optimize: bool,

    /// Print the tokens to stdout
    #[arg(short = 't', long, global = true)]
    output_tokens: bool,

    /// Print the ast to stdout
    #[arg(short = 'a', long, global = true)]
    output_ast: bool,

    /// Print the Internal ir to stdout
    #[arg(short = 'i', long, global = true)]
    output_ir: bool,

    /// Print the produced LLVM ir to stdout
    #[arg(short = 'l', long, global = true)]
    output_llvm: bool,
}

#[derive(Parser)]
struct CompilerCli {
    #[command(subcommand)]
    command: CompilerCommand,

    #[command(flatten)]
    debug: DebugArguments,
}

fn main() -> anyhow::Result<()> {
    let arguments = CompilerCli::parse();
    let compiler_options = CompilerOptions {
        dont_optimize: arguments.debug.dont_optimize,
        output_tokens: arguments.debug.output_tokens,
        output_ast: arguments.debug.output_ast,
        output_ir: arguments.debug.output_ir,
        output_llvm: arguments.debug.output_llvm,
    };

    match arguments.command {
        CompilerCommand::Run { input_file } => {
            build(input_file, compiler_options).context("Building input file")?;
        }
    }

    Ok(())
}
