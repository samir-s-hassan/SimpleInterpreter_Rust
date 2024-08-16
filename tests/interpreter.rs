extern crate asalang;
extern crate nom;
use std::io::Write;

use asalang::*;
use nom::IResult;

macro_rules! test_fragment {
  ($func:ident, $test:tt, $expected:expr) => (
    #[test]
    fn $func() -> Result<(),AsaErrorKind> {
      let tokens = lex($test);
      match program(tokens) {
        Ok((tokens, tree)) => {
          assert_eq!(tokens.is_done(), true); // Check that input token stream is fully parsed
          let mut interpreter = Interpreter::new();
          let result = interpreter.exec(&tree);
          std::io::stdout().flush();
          assert_eq!(result, $expected);
          Ok(())
        },
        Err(e) => Err(AsaErrorKind::Generic(format!("{:?}",e))),
      }
    }
  )
}

macro_rules! test_program {
  ($func:ident, $test:tt, $expected:expr) => (
    #[test]
    fn $func() -> Result<(),AsaErrorKind> {
      let tokens = lex($test);
      match program(tokens) {
        Ok((tokens, tree)) => {
          assert_eq!(tokens.is_done(), true); // Check that input token stream is fully parsed
          let mut interpreter = Interpreter::new();
          let compile_result = interpreter.exec(&tree)?;
          let main_result = interpreter.start_main(vec![]);
          assert_eq!(main_result, $expected);
          Ok(())
        },
        Err(e) => Err(AsaErrorKind::Generic(format!("{:?}",e))),
      }
    }
  )
}

// Test interpreter fragments (no main function)
test_fragment!(interpreter_numeric, r#"123"#, Ok(Value::Number(123)));
test_fragment!(interpreter_string, r#""helloworld""#, Ok(Value::String("helloworld".to_string())));
test_fragment!(interpreter_bool_true, r#"true"#, Ok(Value::Bool(true)));
test_fragment!(interpreter_bool_false, r#"false"#, Ok(Value::Bool(false)));
test_fragment!(interpreter_identifier, r#"x"#, Err(AsaErrorKind::UndefinedFunction));
test_fragment!(interpreter_function_call, r#"foo()"#, Err(AsaErrorKind::UndefinedFunction));
test_fragment!(interpreter_function_call_one_arg, r#"foo(a)"#, Err(AsaErrorKind::UndefinedFunction));
test_fragment!(interpreter_function_call_more_args, r#"foo(a,b,c)"#, Err(AsaErrorKind::UndefinedFunction));
test_fragment!(interpreter_variable_define, r#"let x = 123;"#, Ok(Value::Number(123)));
test_fragment!(interpreter_variable_init, r#"let x = 1;"#, Ok(Value::Number(1)));
test_fragment!(interpreter_variable_bool, r#"let bool = true;"#, Ok(Value::Bool(true)));
test_fragment!(interpreter_variable_string, r#"let string = "HelloWorld";"#, Ok(Value::String("HelloWorld".to_string())));
test_fragment!(interpreter_variable_init_no_space, r#"let x=1;"#, Ok(Value::Number(1)));
test_fragment!(interpreter_math, r#"1 + 1"#, Ok(Value::Number(2)));
test_fragment!(interpreter_math_no_space, r#"1-1"#, Ok(Value::Number(0)));
test_fragment!(interpreter_math_multiply, r#"2 + 4"#, Ok(Value::Number(6)));
test_fragment!(interpreter_assign_math, r#"let x = 1 + 1;"#, Ok(Value::Number(2)));
test_fragment!(interpreter_assign_function, r#"let x = foo();"#, Err(AsaErrorKind::UndefinedFunction));
test_fragment!(interpreter_assign_function_arguments, r#"let x = foo(a,b,c);"#, Err(AsaErrorKind::UndefinedFunction));

// Test full programs
test_program!(interpreter_define_function, r#"fn main(){return foo();} fn foo(){return 5;}"#, Ok(Value::Number(5)));
test_program!(interpreter_define_function_args, r#"fn main(){return foo(1,2);} fn foo(a,b){return a+b;}"#, Ok(Value::Number(3)));
test_program!(interpreter_define_function_more_statement, r#"fn main() {
  return foo();
}
fn foo(){
  let x = 5;
  return x;
}"#, Ok(Value::Number(5)));
test_program!(interpreter_define_full_program, r#"fn foo(a,b,c) {
  let x = a + 1;     
  let y = bar(c + b); 
  return x + y;
}

fn bar(a) {
  return a + 3;
}

fn main() {
  return foo(1,2,3);  
}"#, Ok(Value::Number(10)));

// SAMIR --------------------> added 5 more tests

test_fragment!(samir_interpreter_identifier_redefinition, r#"let x = 5; let x = x + 1;"#, Ok(Value::Number(6)));
test_fragment!(samir_interpreter_math_subtract, r#"5 - 3"#, Ok(Value::Number(2)));
test_fragment!(samir_interpreter_assign_math_spaces, r#"let    x    =    1    +   1;"#, Ok(Value::Number(2)));
test_fragment!(samir_interpreter_alphanumeric, r#"hello123"#, Err(AsaErrorKind::UndefinedFunction));
test_fragment!(samir_interpreter_variable_false, r#"let bool = false;"#, Ok(Value::Bool(false)));
