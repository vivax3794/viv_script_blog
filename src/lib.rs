use anyhow::Context;

mod parsing;

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

pub fn build(file_name: String, options: CompilerOptions) -> anyhow::Result<()> {
    let code = std::fs::read_to_string(file_name).context("Reading input file")?;

    let ast = parsing::parse(&code, &options).context("Parsing input file")?;

    Ok(())
}
