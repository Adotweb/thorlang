
//let code = ".>.<[+++++++++>+<]";


fn interpret(tape, code){ 

	let current_pointer = 0;



	let current_program_pointer = 0;

	let last_jump_point = 0;


	while(current_program_pointer < code.len()){
		let current_symbol = code[current_program_pointer];

		if(current_symbol == "<"){
			if(current_pointer == 0){
				current_pointer = 0;
			}else{
				current_pointer = current_pointer - 1;
			}
		}	
	
		if(current_symbol == ">"){
			if(current_pointer == tape.len()){
				current_pointer = tape.len();
			}else{
				current_pointer = current_pointer + 1;
			}
		}		
			
		if(current_symbol == "["){
			last_jump_point = current_program_pointer;
		}

		if(current_symbol == "+"){
			let current_space = tape[current_pointer];


			tape[current_pointer] = current_space + 1;

						

			if(tape[current_pointer] == 10){
				tape[current_pointer] = 0;
			}
		}

		if(current_symbol == "."){
			print tape[current_pointer];
		}

		if(current_symbol == ","){
			let input_number = get_input("enter some number");
			
			input_number = input_number[input_number.len() - 1];

			input_number = cast_to(input_number, "number");	
				
			tape[current_pointer] = input_number;
		}


		current_program_pointer = current_program_pointer + 1;
			
		if(current_symbol == "]"){
	
			let current_space = tape[current_pointer];
	


			if(current_space != 0){	
				current_program_pointer = last_jump_point;
			}
		}			

	}

	return tape;
}



let tape = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

let code = ",>,<[+++++++++>+<]>.";


let tape = interpret(tape, code);
