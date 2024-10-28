function fact(n){
	if(n == 0){
		return 1;
	}else{
		return n * fact(n - 1);
	}
}

let time1 = Date.now();
for(let i = 0; i < 100; i++){
	let num = fact(i);	
}
let time2 = Date.now();
console.log(time2 - time1);
