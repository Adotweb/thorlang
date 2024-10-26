let somevalue = 14;


let a = 0;
a = 9;

fn obj(list){


	if(typeOf(list) != "array"){
		throw "list must be an array";
	}

	let iter = 0;

	let obj;

	while(iter < list.len()){
		
		let key_value_pair = list[iter];

		if(typeOf(key_value_pair) != "array"){
			throw "any key value pair must be an array";
		}

		if(key_value_pair.len() != 2){
			throw "any key_value_pair must contain two entries";
		}

		obj[key_value_pair[0]] = key_value_pair[1];

		iter = iter + 1;
	}

	return obj;

}

let o = obj([
	["key", "value"],
	["siuu", obj([
		["key", "value"]
	])]
]);



let a = 4 + "hello";

print a;

return obj;
