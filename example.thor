let module = import("module.thor");

module.vec_overloads();

let a = [0, 1, 2];


let b = [2, 3, 9];

let matrix = [
	[1, 0, 0],
	[0, 2, 0],
	[0, 0, 1]
];

//+ treated as a unary operation acts as an abs function now
print +3;

//matrix * vector multiplication works as well as vector addition and scalar multiplication (left and right commutative)
print 3 * (matrix * (a + b) * 4)
