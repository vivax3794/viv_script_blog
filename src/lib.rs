use anyhow::Context;

mod code_gen;
mod ir;
mod parsing;
mod type_analyzer;

type IntType = i32;
type FloatType = f64;

const IntWidth: usize = 32;
const FloatWidth: usize = 64;

pub struct CompilerOptions {
    pub dont_optimize: bool,
    pub output_tokens: bool,
    pub output_ast: bool,
    pub output_ir: bool,
    pub output_llvm: bool,
}

pub fn build(file_name: &str, output_file: &str, options: CompilerOptions) -> anyhow::Result<()> {
    let code = std::fs::read_to_string(file_name).context("Reading input file")?;

    let ast = parsing::parse(&code, &options).context("Parsing input file")?;
    let ir = type_analyzer::Analyzer::new()
        .resolve_module(&ast)
        .context("Resolving types")?;

    if options.output_ir {
        println!("{ir:#?}");
    }

    let inkwell_context = inkwell::context::Context::create();
    let mut code_gen = code_gen::CodeGen::new(&inkwell_context);
    code_gen.compile_module(&ir);

    let llvm_ir_output_file = temp_file::empty();
    code_gen.output_to_file(llvm_ir_output_file.path(), &options);

    let object_file = temp_file::empty();
    compile_to_objectfile(
        llvm_ir_output_file.path().to_str().unwrap(),
        object_file.path().to_str().unwrap(),
    )?;
    compile_to_binary(object_file.path().to_str().unwrap(), output_file)?;

    Ok(())
}

fn find_on_path(program: &str) -> Option<std::path::PathBuf> {
    let path = std::env::var_os("PATH")?;

    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(program);

        if candidate.is_file() {
            return Some(candidate);
        }
    }

    None
}

fn find_any_on_path(programs: Vec<&str>) -> Option<std::path::PathBuf> {
    for program in programs {
        if let Some(path) = find_on_path(program) {
            return Some(path);
        }
    }

    None
}

fn compile_to_objectfile(from: &str, to: &str) -> anyhow::Result<()> {
    let clang = find_on_path("llc").ok_or(anyhow::anyhow!("Llc not found on path"))?;
    std::process::Command::new(clang)
        .args([from, "-filetype=obj", "-o", to])
        .spawn()?
        .wait()?
        .success()
        .then_some(())
        .ok_or(anyhow::anyhow!("Llc failed"))?;

    Ok(())
}

fn compile_to_binary(from: &str, to: &str) -> anyhow::Result<()> {
    let clang = find_any_on_path(vec!["clang", "gcc"])
        .ok_or(anyhow::anyhow!("Clang or gcc not found on path"))?;
    std::process::Command::new(clang)
        .args([from, "-no-pie", "-o", to])
        .spawn()?
        .wait()?
        .success()
        .then_some(())
        .ok_or(anyhow::anyhow!("Clang/gcc failed"))?;

    Ok(())
}
