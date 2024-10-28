
//let code = ".>.<[+++++++++>+<]";


fn interpret(tape, code){

	
	let program_pointer = 0;

	//is increased whenever whenever we encounter [ so we move down a nesting	
	let loop_pointer = 0;


	//the length of this is the stack we cannot go deeper than 7 nestings at the moment
	let jump_points = [0, 0, 0, 0, 0, 0, 0];


	let tape_pointer = 0;


	while(program_pointer < code.len()){
		

		let symbol = code[program_pointer];


		if(symbol == "<"){
			if(tape_pointer == 0){
				tape_pointer == 0;
			}else{
				
				tape_pointer = tape_pointer - 1;
			}
		}
		if(symbol == ">"){
			if(tape_pointer == tape.len()){
				tape_pointer == tape.len();
			}else{
				tape_pointer = tape_pointer + 1;
			}
		}
		if(symbol == "."){
			print tape[tape_pointer];
		}
		if(symbol == ","){
			let input = get_input("input some number");
			
			//since only numbers 0 to 9 are allowed we only need the last digit.

			let number = cast_to(input, "number");
			
			tape[tape_pointer] = number;
			
		}
		if(symbol == "["){

			//if the current symbol is 0 we skip the loop

			if(tape[tape_pointer] != 0){
				jump_points[loop_pointer] = program_pointer;	
				loop_pointer = loop_pointer + 1;	
			}else{
				//represents the numbers of ] we need to find until were done.	(increased by one when we encounter [)
				let done = 1;
				
				let iter = program_pointer + 1;
				
				while(done != 0){
					let symbol = code[iter];
						
					if(symbol == "]"){
						done = done - 1;
					}
					if(symbol == "["){
						done = done + 1;
					}
						
					iter = iter + 1;
				}	
				
				program_pointer = iter - 2;
			}

		}
	
		if(symbol == "+"){
				
			tape[tape_pointer] = tape[tape_pointer] + 1;
			
			if(tape[tape_pointer] == 256){
				tape[tape_pointer] = 0;
			}
		}
		if(symbol == "-"){
			tape[tape_pointer] = tape[tape_pointer] - 1;

			if(tape[tape_pointer] == -1){
				tape[tape_pointer] = 255;
			}
		}
		
		program_pointer = program_pointer + 1;

		

		if(symbol == "]"){

			if(tape[tape_pointer] != 0){	
				loop_pointer = loop_pointer - 1;
				program_pointer = jump_points[loop_pointer];			
			}else{
				loop_pointer = loop_pointer - 1;
			}
		}	
	}

	return tape;
}




let tape = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

let code = ",[+.]";

let tape = interpret(tape, code);

print tape;
