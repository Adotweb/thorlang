//import .so files
let s = import_lib("liblib.so");

s.start_window();

overload + (a, b){
		

	let string1 = cast_to(a, "string");
	let string2 = cast_to(b, "string");


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
