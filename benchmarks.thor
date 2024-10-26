fn fact(n){
	if(n == 0){
		return 1;
	}else{
		return n * fact(n - 1);
	}

}

fn fib(n){
	if(n == 1){
		return 1;
	}
	if(n == 0){
		return 0;
	}else{	
		return fib(n - 1) + fib(n - 2);
	}
}



let now = get_now();


print now;

let iter = 0;
while(iter < 21){

	

	print "iteration:";
	print iter;
	print fib(iter);
	
	iter = iter + 1;
}

let now2 = get_now();


print "";
print "time since";
print now2 - now;
