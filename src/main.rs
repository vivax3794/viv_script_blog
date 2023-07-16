use anyhow::Context;
use clap::{Args, Parser, Subcommand};

use viv_script::{build, CompilerOptions};

#[derive(Subcommand)]
enum CompilerCommand {
    /// Compile and run file
    Run { input_file: String },
    /// Compile file
    Build {
        input_file: String,
        output_file: String,
    },
    /// Compile and run all files in integration_tests/
    Test,
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

fn run_test(file: &str) -> anyhow::Result<()> {
    print!("Running test: {file} ... ");

    let output_file = temp_file::empty();
    build(
        file,
        output_file.path().to_str().unwrap(),
        CompilerOptions {
            dont_optimize: false,
            output_tokens: false,
            output_ast: false,
            output_ir: false,
            output_llvm: false,
        },
    )?;
    let output = std::process::Command::new(output_file.path())
        .spawn()?
        .wait()?;

    if !output.success() {
        println!("ERROR");
        Err(anyhow::anyhow!("Test failed: {}", file))?;
    } else {
        println!("OK");
    }

    Ok(())
}

fn run_tests() -> anyhow::Result<()> {
    for file in std::fs::read_dir("integration_tests")? {
        let file = file?;
        let file_name = file.file_name().into_string().unwrap();
        if file_name.ends_with(".viv") {
            run_test(file.path().to_str().unwrap())?;
        }
    }

    Ok(())
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
            let output_file = temp_file::empty();
            build(
                &input_file,
                output_file.path().to_str().unwrap(),
                compiler_options,
            )
            .context("Building input file")?;

            let output = std::process::Command::new(output_file.path())
                .spawn()?
                .wait()?;

            std::process::exit(output.code().unwrap_or(1));
        }
        CompilerCommand::Build {
            input_file,
            output_file,
        } => {
            build(&input_file, &output_file, compiler_options).context("Building input file")?;
        }
        CompilerCommand::Test => run_tests()?,
    }

    Ok(())
}
