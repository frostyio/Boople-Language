pub mod lexer;

#[cfg(test)]
mod tests {
    use std::fs;
	use super::lexer;

	#[test]
    fn lexical_test() {
		let hello_world = fs::read_to_string("../examples/hello_world.bop").expect("unable to find exmaple file");
		let _ = lexer::analyze_chunk(hello_world);
	}
}
