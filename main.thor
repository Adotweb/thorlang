let module = import("module.thor");

//overload vector operations
module.vec_overloads();


let a = [0, 1, 2];

let matrix = [
	[1, 0, 0],
	[0, 2, 0],
	[0, 0, 1]
];


print 3 * (matrix * a);
