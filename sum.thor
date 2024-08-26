let obj = import("module.thor");


let arr = [1, 2, 3];

print arr[0];

let p = try {
	return arr[0];
};



overload + (a, b) {
	return a[0] + b[2];
}

let arr1 = [0, 1, 2];
let arr2 = [0, 2, 3];

let s = arr1 + arr2;

print s;
