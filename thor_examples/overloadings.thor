
//funciton that initialized overloadings in the scope of execution
//means we can export it to another file
fn init_overloadings(){

	//+ for string formatting
	overload + (a, b){

		let s1 = cast_to(a, "string");
		let s2 = cast_to(b, "string");
	

		return s1 + s2;
	}

}


return init_overloadings;
