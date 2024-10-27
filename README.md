# Thorlang 

## Overview 

I build Thorlang (or thor for short) to understand the steps necessary in order to build a interpreted language from the ground up. The endgoal of the project is to have a working programming language that is easy to learn and somewhat efficient... this will take a long time though... 

## Readme info

This project is written and maintained in the scope of a Maturapaper.
**note**
Thorlang is highly experimental and still in development, using it in production is not recommended.


## Installation 
To install Thorlang you can either use the install.sh shell script (only on linux). Or you can compile thorlang using src : 
`git clone https://github.com/Adotweb/thorlang`
`cd thorlang`

then you can run your code directly through cargo run 
`cargo run run [your filename]`

or you can build an executable using cargo build
`cargo build`

this will put an executable named `thorlang` into the target/debug/ folder

this executable can now be used as the thorlang runtime

`[path/to/thorlang_executable] run [path/to/file.thor]`

## API Reference

#### Variables
To declare a variable simply put let in front of it.


```thor
  let variable = 0;
  //this is a comment
  //reassignment
  variable = 1;
```

The primitives in Thorlang are 

|Primitive|Explanation|
|--|--|
|String|Anything between ""|
|Number| float of 64 bits|
|Bool|true or false|
|Array|any of the former between [] and seperated by commas|
|Object| variable that have fields of the former types accessed by strings|
|Nil| is just `nil`|

To declare an object initialize a variable and put the fields on it. 

### strings and arrays

strings and array can be accessed using brackets like this: 

```thor
    let string = "hello";

    let array = ["a", "b"];

    print string[0]; //prints "h"

    print array[1]; //prints "b"
```

```thor
  let obj;
  obj.field = 0;
  //or use the dynamic field operator
  obj["field"] = 0;

  //to get the field do this
  print obj["field"];
  //or
  print obj.field;

```


#### Functions
To declare a function use the fn keyword. Use the return keyword to return a value. 


```thor
  fn function_name(argument){
    //dosomething...
    return argument;
  }
```
> **Note** 
Functions in thor are pure by default, so they cannot change variables beyond of their scope.

```thor
let a = 0;

fn something(init){
  a = init;
  print a; //<- prints init
}

print a; //<- prints 0;
```

#### Control flow
If and else statements are very similar to every other language.
While loops are the only loop structure available in thorlang.

```thor
if (condition){
  //do something
} else {
  //do something else
}

while (condition){
  //do something
}
```

#### Try expressions and isError 

Sometimes we want to run something that might throw, try expressions (yes expressions because they are superior) allow you to do that: 

**Note** Since try blocks are expressions, after any try block there needs to be a `; semicolon`.
```thor

let maybeanerror = try {
    
    let array = [0, 1, 2, 3];

    return array[0];
};

//prints 0
print maybeanerror;

let maybeanerror2 = try {
    
    let array = [0, 1, 2, 3];

    return array[4];
};

//prints EvalError
print maybeanerror2;

//prints true
print isError(maybeanerror2);
```

#### Operator Overloading

Thor allows operator overloading, to overload an operator do the following: 
```thor 

overload + (a, b) {
    return a[1] + b[1]
}

let a = [0, 1];
let b = [0, 1];

//prints 2
print a + b;

```

when the code inside the overload (the operation) fails the operation will go to default behaviour.

**Note** The number of arguments in an overload determines whether it will be used as the arity of the operation.
This means that to overload the (numerical) number operator you just put a inside the parenthesis : 

```thor 

overload - (a) {
    return a[0]
}

let a = [0, 1];

//prints 0
print -a;

```


#### Modules

Thorlang supports modules, to import a module use the import function:

```thor
//main.thor
let something = import("module.thor");

print something;

```

To export something use the return statement at the end of a thor file: 

```thor
//module.thor
let something = 10;


return something;
```

#### Native functions

Of course thorlang has some native functions (this list will be expanded):

| Functions | arguments     | Description                       |
| :-------- | :------- | :-------------------------------- |
| `printf`      | `thing` | prints `thing` (this is the functional representation of the print statement)|
| `get_input` | `any value : message` | prints the message to the terminal and accepts an input (text) which it returns | 
| `getTime` | No arguments | returns the current unix time (unimplemented)| 
| `import` | `String : Namespace` | returns the returned value of the given file and throws if the file does not exist| 
| `isError` | any value | returns true if the argument provided is an error and false else| 
| `get_now` | No arguments | returns the current unix time in milliseconds | 
| `cast_to` | any value, string : type | tries to cast the first value to the type thats provided through a string | 

#### Native Methods

And of course Thorlangs native types have methods on them to make your life easier, some of them are listed here (this list will be expanded):

|Type | method | arguments | Description|
|---|---|---|---|
|Number| sqrt | none | returns the square root of the number the method was called on| 
|Array | len | none | returns the length of the array the method was called on |
|Array | push | value | pushes the value to the array and returns the new array|

## Roadmap 

- [x] turing completeness
- [ ] removing all weird bugs/making everything stable
- [ ] removing bad code/pretty- and smartifying everything ive writte so far
- [ ] better errors and error handling
- [ ] easifying installation and documentation
- [ ] std library (for example weblib)
