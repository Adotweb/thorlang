let p = import("module.thor");


p.count(10);

let obj;

obj.array = [0, 1, 2, 3];


print obj;

fn whileTest(n){

	let i = 0;	
	while (i < n){
		if (i == 4){
			return i;
		}
		print i;
		i = i + 1;
	}
}

