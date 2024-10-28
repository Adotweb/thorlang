function fib(n) {

	if(n == 0){
		return 0;
	}if(n == 1){
		return 1;
	}else{
		return fib(n - 1) + fib(n - 2);
	}
}

let time1 = Date.now();
for(let i = 0; i < 30; i++){
	console.log(fib(i));
}

let time2 = Date.now();

console.log(time2 - time1);
