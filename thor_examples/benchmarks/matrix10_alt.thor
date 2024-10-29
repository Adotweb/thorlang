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

overload * (a, b){

	if(a.type == "vec"){
		if(b.type == "vec"){
			let iter = 0;
				
			let ret = a;

			while(iter < a.values.len()){
				ret.values[iter] = a.values[iter] * b.values[iter];


				iter = iter + 1;
			}

			return ret;
		}
	}	
	throw "";
}

//can ba used liket this: 
// A_[4, 5] gives the element at the 4th row 5th column
// when one of the array entries is "i" then we return the entire column or entire row as a vector
overload ^ (a, b){
	print a;
	print b;

	if(a.type == "matrix"){

		let i = b[0];
		let j = b[1];

		let res;
		
		if(j == "i"){
			res.type = "vec";
			res.values = a.values[i];
			return res;
		}

		if(i == "i"){
				
			res.type = "vec";
			res.values = a;
			let iter = 0;
			while(iter < res.values.len())	{
				
				res.values[iter] = a.values[iter][i];	
				
				iter = iter + 1;
			}
				
			return res;
		}


		return a[i][j];
	}

}


overload * (a, b){
	
	if(a.type == "mat"){
		
		if(b.type == "mat"){
			
			let i = 0;
			while(i < b.len()){

				let row = mat[i];
				let j = 0;

				while(j < row.len()){
			
					

					j = j + 1;
				}
				i = i + 1;
			}

		}

	}

}

let I = m([
	[1, 0],
	[0, 1]
]);

overload ^ (a, b){
	print a;
	print b;
	return a + b;
}

let five = 4^5;
