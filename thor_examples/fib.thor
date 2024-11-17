let value = 0;
//print value;

fn mute_value(){
	value = 4;	
}

mute_value();

//print value;

fn fact(n){

	if(n == 0){
		return 0;
	}else{
		print n;
		let p = n * fact(n - 1);

		return p;
	}

}

fn make_counter(init){

	let count = init;

	fn mute(func){
		count = func(count);
	}

	fn get(){
		return count;
	}


	let ret;
	ret.mute = mute;
	ret.get = get;
	return ret;
}

let counter = make_counter();



fn add_one(x){
	return x + 1;
}

fn subtract_one(x){
	return x - 1;
}

counter.mute(add_one);
print counter.get();
counter.mute(subtract_one);
print counter.get();
