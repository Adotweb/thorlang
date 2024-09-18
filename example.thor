let somevalue = import("module.thor");

printf(somevalue);

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

printf(ølist);

fn makeCounter(){
	let count = 0;	
	fn inc(){
		count = count + 1;
		printf(count);
	}
	
	return inc;
}


let inc = makeCounter();

inc();
inc();
inc();
