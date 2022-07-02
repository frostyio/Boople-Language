pub mod lexer;
pub mod ast;

#[cfg(test)]
mod tests {
    use std::fs;
	use super::{lexer, ast};

	#[test]
    fn lexical_test() {
		let hello_world = fs::read_to_string("../examples/hello_world.bop").expect("unable to find exmaple file");
		let lexed = lexer::analyze_chunk(hello_world);
		match lexed {
			Ok(_result) => {
				// println!("{:#?}", result)
			},
			Err(err) => panic!("{}", err)
		}
	}

	#[test]
	fn ast_test() {
		let hello_world = fs::read_to_string("../examples/hello_world.bop").expect("unable to find exmaple file");
		let lexed = match lexer::analyze_chunk(hello_world) {
			Ok(result) => result,
			Err(err) => panic!("{}", err)
		};

		let ast = ast::create_ast(lexed);
		// println!("{:#?}", ast.tree.borrow());
	}
}
