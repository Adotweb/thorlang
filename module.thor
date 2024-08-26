let obj;


fn vec_overloads(){


	//vector addition
	overload + (a, b) {
		
		let vec = a;

		if(a.len() != b.len()){
			//just do something that throws
			print "cannot add vectors of different rangs";
			return nil;
		}

		let i = 0;
		while(i < a.len()){

			vec[i] = vec[i] + b[i];
			i = i + 1;
		}


		return vec;
	}

	//scalar multiplication
	overload * (a, b) {
		
		let result = try {
			let vec = b;
			let i = 0;
			while(i < vec.len()){
				vec[i] = vec[i] * a;
				i = i + 1;
			}

			return vec;
		};

		if(!isError(result)){
			return result;
		}

		let vec = a;
		let i = 0;
		while(i < vec.len()){
			vec[i] = vec[i] * b;
			i = i + 1;
		}
		return vec;
	}

}

obj.vec_overloads = vec_overloads;

return obj;
