let p = import("module.thor");

print p;

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

fn while_test(){
	let i = 0;	
	while (i < 10){
		if(i > 5){
			return i;
		}

		i = i + 1;
	}

}


let obj = generateObj([
	["key1", "value1"],
	["key2", nil]
]);


print while_test();

return obj;
