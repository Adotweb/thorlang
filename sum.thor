
fn Try(a){
	
	let result = try{
		return a;
	};

	if(true){
		return a;
	}

	return result;
}

overload + (a, b) {	
	let vec1 = a;


	let i = 0;
	while(i < a.len()){
		vec1[i] = vec1[i] + b[i];
		i = i + 1;
			
	}

	return vec1;
}

overload * (a, b){


	let triedone = try {

		let vec = b;
		let i = 0;
		while(i < b.len()){
			vec[i] = vec[i] * a;
			i = i + 1;
		}

		return vec;
	};
		
	if(!isError(triedone)){
		return triedone;
	} 

	let triedtwo = try{
		let vec = a;
		let i = 0;
		while(i < a.len()){
			vec[i] = vec[i] *b;
			i = i + 1;
		}

		return vec;
	};

	if(!isError(triedtwo)){
		return triedtwo;
	}
}


let a = [1, 0, 0];
let b = [0, 1, 0];



print a + b;
