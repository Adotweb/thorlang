fn generateObj(arr){

	let i = 0;

	let obj;

	while(i < arr.len()){
			
		let field = arr[i];	

		obj[field[0]] = field[1];

		i = i + 1;
	}

	return obj;
}


let obj = generateObj([
	["key1", "value1"],
	["key2", nil]
]);


print obj;
