overload + (a, b){
	
	let string1 = cast_to(a, "string");
	let string2 = cast_to(b, "string");

	return string1 + string2;
}

fn fib(n){
	
	if(n == 0){
		return 0;
	}
	if(n == 1){
		return 1;
	}
	else{
		return fib(n - 1) + fib(n - 2);
	}
}

let unix_time_start = get_now();

let iter = 0;

while(iter < 30){

	print "iteration " + iter + ": fib(" + iter + ") = " + fib(iter);

	iter = iter + 1;
}

let unix_time_stop = get_now();

let duration = unix_time_stop - unix_time_start;

print "took " + duration + " ms to complete this task";
