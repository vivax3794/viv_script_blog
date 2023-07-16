use crate::{ir, CompilerOptions};
use inkwell::{context::Context, IntPredicate};

pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: inkwell::module::Module<'ctx>,
    builder: inkwell::builder::Builder<'ctx>,
    fpm: inkwell::passes::PassManager<inkwell::module::Module<'ctx>>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        let module = context.create_module("main");
        let builder = context.create_builder();
        let fpm = inkwell::passes::PassManager::create(());

        fpm.add_ipsccp_pass();
        fpm.add_new_gvn_pass();
        fpm.add_ind_var_simplify_pass();
        fpm.add_instruction_simplify_pass();
        fpm.add_instruction_combining_pass();

        fpm.add_constant_merge_pass();
        fpm.add_global_optimizer_pass();

        fpm.add_demote_memory_to_register_pass();
        fpm.add_merge_functions_pass();
        fpm.add_dead_arg_elimination_pass();
        fpm.add_function_attrs_pass();
        fpm.add_function_inlining_pass();
        fpm.add_tail_call_elimination_pass();

        fpm.add_licm_pass();
        fpm.add_cfg_simplification_pass();

        fpm.add_global_dce_pass();
        fpm.add_aggressive_dce_pass();
        fpm.add_loop_deletion_pass();

        Self {
            context: &context,
            module,
            builder,
            fpm,
        }
    }

    fn int_type(&self) -> inkwell::types::IntType {
        use crate::IntWidth;

        match IntWidth {
            8 => self.context.i8_type(),
            16 => self.context.i16_type(),
            32 => self.context.i32_type(),
            64 => self.context.i64_type(),
            128 => self.context.i128_type(),
            _ => panic!("Invalid int width"),
        }
    }

    fn compile_libc_definitions(&mut self) {
        let i32_type = self.context.i32_type();
        let i8_type = self.context.i8_type();
        let i8_ptr_type = i8_type.ptr_type(inkwell::AddressSpace::default());
        let void_type = self.context.void_type();

        let printf_type = i32_type.fn_type(&[i8_ptr_type.into()], true);
        self.module.add_function("printf", printf_type, None);

        let abort_type = void_type.fn_type(&[], false);
        self.module.add_function("abort", abort_type, None);
    }

    fn compile_int_expression(&self, expression: &ir::IntExpression) -> inkwell::values::IntValue {
        match expression {
            ir::IntExpression::Literal(int) => self.int_type().const_int(*int as u64, false),
        }
    }

    fn compile_comparison(
        &self,
        comparison: &ir::ComparisonExpression,
    ) -> inkwell::values::IntValue {
        match comparison {
            ir::ComparisonExpression::IntComparison(left, op, right) => {
                let left = self.compile_int_expression(left);
                let right = self.compile_int_expression(right);

                let op = match op {
                    ir::IntComparisonOp::Equal => IntPredicate::EQ,
                };

                self.builder
                    .build_int_compare(op, left, right, "Comparison")
            }
        }
    }

    fn compile_bool_expression(
        &self,
        expression: &ir::BooleanExpression,
    ) -> inkwell::values::IntValue {
        match expression {
            ir::BooleanExpression::Literal(boolean) => {
                self.context.bool_type().const_int(*boolean as u64, false)
            }
            ir::BooleanExpression::Comparison(comparison) => self.compile_comparison(comparison),
        }
    }

    fn compile_print_statement(&self, statement: &ir::PrintStatement) {
        let printf = self.module.get_function("printf").unwrap();

        let format_string = match statement {
            ir::PrintStatement::Int(_) => "%d\n",
            ir::PrintStatement::Boolean(_) => "Bool(%d)\n", // This isnt the best way to do this
        };
        let format_string = self
            .builder
            .build_global_string_ptr(format_string, "format_string")
            .as_pointer_value();

        match statement {
            ir::PrintStatement::Int(int_expression) => {
                let int_value = self.compile_int_expression(int_expression);
                self.builder.build_call(
                    printf,
                    &[format_string.into(), int_value.into()],
                    "printf",
                );
            }
            ir::PrintStatement::Boolean(boolean_expression) => {
                let boolean_value = self.compile_bool_expression(boolean_expression);
                self.builder.build_call(
                    printf,
                    &[format_string.into(), boolean_value.into()],
                    "printf",
                );
            }
        }
    }

    fn compile_const_printf(&self, msg: &str) {
        let printf = self.module.get_function("printf").unwrap();
        let format_string = self
            .builder
            .build_global_string_ptr(msg, "Const_Print")
            .as_pointer_value();

        self.builder
            .build_call(printf, &[format_string.into()], "Const_Print");
    }

    fn compile_assert(&self, expression: &ir::BooleanExpression, message: &Option<String>) {
        let condition_value = self.compile_bool_expression(expression);

        let current_block = self.builder.get_insert_block().unwrap();
        let fail_block = self
            .context
            .insert_basic_block_after(current_block, "fail_block");
        let continue_block = self
            .context
            .insert_basic_block_after(fail_block, "continue_block");

        self.builder
            .build_conditional_branch(condition_value, continue_block, fail_block);
        self.builder.position_at_end(fail_block);

        if let Some(message) = message {
            self.compile_const_printf(&format!("Assert failed: {message}\n"));
        } else {
            self.compile_const_printf("Assert failed\n");
        }

        let abort = self.module.get_function("abort").unwrap();
        self.builder.build_call(abort, &[], "Assert_Fail_Exit");
        self.builder.build_unreachable();

        self.builder.position_at_end(continue_block);
    }

    fn compile_statement(&self, statement: &ir::Statement) {
        match statement {
            ir::Statement::Print(print_statement) => self.compile_print_statement(print_statement),
            ir::Statement::Assert(expression, message) => self.compile_assert(expression, message),
        }
    }

    fn compile_top_level_statement(&self, statement: &ir::ToplevelStatement) {
        match statement {
            ir::ToplevelStatement::MainFunction(statements) => {
                let i32_type = self.context.i32_type();
                let main_type = i32_type.fn_type(&[], false);
                let main = self.module.add_function("main", main_type, None);
                let main_block = self.context.append_basic_block(main, "entry");
                self.builder.position_at_end(main_block);

                for statement in statements {
                    self.compile_statement(statement);
                }

                self.builder
                    .build_return(Some(&i32_type.const_int(0, false)));
            }
        }
    }

    pub fn compile_module(&mut self, module: &ir::Module) {
        self.compile_libc_definitions();

        for statement in &module.0 {
            self.compile_top_level_statement(statement);
        }
    }

    pub fn output_to_file(&mut self, file_path: &std::path::Path, options: &CompilerOptions) {
        if !options.dont_optimize {
            self.fpm.run_on(&self.module);
        }

        self.module.write_bitcode_to_path(file_path);

        if options.output_llvm {
            self.module.print_to_stderr();
        }
    }
}
