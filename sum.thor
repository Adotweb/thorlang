let obj = import("module.thor");


let arr = [1, 2, 3];

print arr[0];

let p = try {
	return arr[4];
};

print p;

overload + (a, b) {

	return a + b;

}
