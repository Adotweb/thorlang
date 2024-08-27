let obj;


fn vec_overloads(){


	overload * (mat, vec){



		//if mat is not a matrix (i.e.) the elements of mat dont have the .len() method

		if(typeOf(mat) != "array"){
			return throw("matrix is not an array");
		}	
		if(typeOf(vec) != "array"){
			return throw("vector has to be an array");
		}
		

		let i = 0;
		while(i < mat.len()){
			if(typeOf(mat[i]) != "array"){
			
				return throw("matrix rows have to be arrays");
			}	
			i = i + 1;
		}
	

		let ret = mat;

		i = 0;
		while(i < mat.len()){
			
			let row = mat[i];

			if(row.len() != vec.len()){
				return throw("");
			}

			let product_sum = 0;

			let j = 0;
			while(j < row.len()){
				
				product_sum = product_sum + row[j] * vec[j];		

				j = j + 1;
			}

			ret[i] = product_sum;

			i = i + 1;
		}
			return ret;



	}

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
