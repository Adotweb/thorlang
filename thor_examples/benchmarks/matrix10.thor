//overload + for string interpolation;
overload + (a, b){
	
	let string1 = cast_to(a, "string");
	let string2 = cast_to(b, "string");

	return string1 + string2;
}


//two random matrices with 10 rows and 10 columns
let mat1 = [[0, 58, 51, 82, 6, 97, 96, 69, 9, 94],
 [59, 45, 99, 63, 76, 98, 47, 87, 58, 78],
 [83, 65, 42, 79, 10, 94, 77, 21, 8, 32],
 [55, 77, 39, 68, 23, 8, 86, 86, 18, 13],
 [40, 7, 48, 45, 86, 6, 42, 21, 52, 84],
 [95, 65, 20, 98, 71, 98, 85, 65, 29, 57],
 [86, 21, 26, 0, 66, 74, 60, 70, 19, 14],
 [41, 22, 60, 42, 38, 41, 58, 30, 39, 69],
 [82, 6, 22, 45, 25, 59, 10, 66, 14, 29],
 [53, 76, 73, 97, 52, 59, 31, 26, 99, 77]];

let mat2 = [[17, 62, 89, 10, 12, 49, 48, 69, 21, 83],
 [44, 61, 77, 42, 8, 32, 31, 0, 28, 72],
 [50, 59, 19, 9, 85, 94, 93, 38, 59, 85],
 [89, 26, 50, 7, 98, 39, 65, 35, 81, 42],
 [87, 73, 51, 58, 66, 61, 30, 77, 75, 20],
 [68, 75, 94, 88, 17, 46, 14, 70, 68, 65],
 [22, 28, 2, 65, 31, 71, 2, 54, 78, 24],
 [56, 29, 29, 62, 89, 71, 60, 13, 56, 47],
 [56, 99, 33, 89, 95, 83, 79, 93, 19, 6],
 [92, 90, 19, 28, 98, 49, 83, 63, 30, 84]];



//simple matrix multiplication algorithm for square matrices
fn matrix_mult(m1, m2){

	let unix_time1 = get_now();

	let res = m2;

	let i = 0;
	while(i < m1.len()){
		let row = m1[i];

		let j = 0;
		while(j < row.len()){

			
			let w = 0;
			let sum = 0;
			while(w < row.len()){
				
				sum = sum + m1[i][w] * m2[w][j];

				w = w + 1;
			}

			res[i][j] = sum;
			
			j = j + 1;
		}
		i = i + 1;
	}	
	
	let unix_time2 = get_now();

	print "matrix multiplication took " + (unix_time2 - unix_time1) + "ms";

	return res;
}

let res_mat = matrix_mult(mat1, mat2);


//print res_mat;
