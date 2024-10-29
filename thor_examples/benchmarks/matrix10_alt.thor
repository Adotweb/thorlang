fn v(a){

	let ret;
	ret.type = "vec";

	ret.values = a;
	
	return ret;
}
fn m(a){

	let ret;
	ret.type = "mat";

	ret.values = a;
	
	return ret;
}

//dot product
overload * (a, b){
	if(a.type == "vec"){
		if(b.type == "vec"){
			let iter = 0;
			let sum = 0;	
			let ret = a;


			while(iter < a.values.len()){
				sum = a.values[iter] * b.values[iter] + sum;

				
				iter = iter + 1;
			}

			return sum;
		}
	}	
	throw "";
}

//can be used like this: 
// A&[4, 5] gives the element at the 4th row 5th column
// when one of the array entries is "i" then we return the entire column or entire row as a vector
// essentially works like  A_54 in pure math

overload & (a, b){

	if(a.type == "mat"){

		let i = b[0];
		let j = b[1];

		

		let res;
		
		if(j == "i"){
			res.type = "vec";
			res.values = a.values[i];
			return res;
		}

		if(i == "i"){
			let res;	
			res = a;
			res.type = "vec";
			let iter = 0;


			while(iter < res.values.len())	{
				
				res.values[iter] = a.values[iter][j];
				
				
				iter = iter + 1;
			}
				
			return res;
		}

		return a.values[i][j];
	}

}

//naive matrix multiplication for square matrices using operator overloading and instantiator functions
overload * (a, b){
	
	if(a.type == "mat"){
		
		if(b.type == "mat"){
			let res = a;
			
			let i = 0;
			while(i < a.values.len()){
					
				//get the ith row vector
				let row = a&[i, "i"];
			

				let j = 0;
				while(j < row.values.len()){
					//get the jth column vector
					let col = b&["i", j];
				

					//use already defined dot product to multiply the row and column vector 
					let dot = row * col;

					res.values[i][j] = dot;
								
					j = j + 1;
				}
				i = i + 1;
			}

			return res;

		}

	}

}


let A = m([
	[1, 2],
	[3, 4]
]);

let I = m([
	[1, 0],
	[0, 1]
]);

print I * A * I;
