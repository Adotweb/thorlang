//import .so files
//let s = import_lib("liblib.so");

//s.start_window();


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

