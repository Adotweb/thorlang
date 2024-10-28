overload + (a, b){
	
	let string1 = cast_to(a, "string");
	let string2 = cast_to(b, "string");

	return string1 + string2;
}

fn fact(n){
	if(n == 0){
		return 1;
	}else{
		return n * fact(n - 1);
	}
}

let unix_time_start = get_now();

let iter = 0;

while(iter < 25){

	print "iteration " + iter + ": fact(" + iter + ") = " + fact(iter);

	iter = iter + 1;
}

let unix_time_stop = get_now();

let duration = unix_time_stop - unix_time_start;

print "took " + duration + " ms to complete this task";
