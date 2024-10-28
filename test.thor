overload + (a, b){

	let string1 = cast_to(a, "string");
	let string2 = cast_to(b, "string");


	return string1 + string2;
}


print 4 + "hello";
