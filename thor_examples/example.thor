let init_obj = import("module.thor");

let init_overloadings = import("overloadings.thor");

init_overloadings();

let obj = init_obj([
	["key", "value"]
]);

print obj + "hello";


//average operator for arrays
overload ø (a){
	if(typeOf(a) == "array"){

		let i = 0;
		let sum = 0;	
			
		while(i < a.len()){
		
			sum = sum + a[i];	

			i = i + 1;
		}


		return sum/a.len();
	}

	throw "error";
}


let s = 0;

let list = [0, 4, 5, 67];

//prints the average of list
print ølist;


fn makeCounter(){
	let count = 0;	
	fn inc(){
		count = count + 1;
		printf(count);
	}
	

	// these two functions dont reference the same environment, because they have different closures.
	// this will maybe change in the future, so closures are more of a reference thing pointing
	// to the same thing if they are evaluated in the same context, making things both more efficient and smarter

	fn dec(){
		count = count - 1;
		printf(count);
	}

	let ret;

	ret.inc = inc;
	ret.dec = dec;

	return ret;
}


let c = makeCounter();

c.inc();
c.inc();
c.inc();

c.dec();
