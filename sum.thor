overload ? (a, b) {	
	let vec1 = a;


	let i = 0;
	while(i < a.len()){
		vec1[i] = vec1[i] + b[i];
		i = i + 1;
			
	}

	return vec1;
}

overload & (a) {
	let i = 0;
	let vec = a;
	while(i < a.len()){
		vec[i] = -vec[i];
		i = i + 1;
	}

	return vec;
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

	let vec = a;
	let i = 0;
	while(i < a.len()){
		vec[i] = vec[i] *b;
		i = i + 1;
	}

	return vec;
	
}


let a = [1, 0, 0];
let b = [0, 1, 0];



print &(a ? b);
