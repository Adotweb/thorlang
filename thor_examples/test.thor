//import .so files
let s = import_lib("liblib.so");


s.start_window();

let p = [0, 1, 2];

p.push(4);

print p;

overload + (a, b){
		

	let string1 = stringify(a);
	let string2 = stringify(b);


	return string1 + string2;
}


//new overload can use old overloaded operators
overload + (a, b){
	
	print a;
	print b;

	return a + b;

}
print 4 + "hello";


while(true){
	s.send_message(4);
}
