# Thorlang 

## Overview 

I build Thorlang (or thor for short) to understand the steps necessary in order to build a interpreted language from the ground up. The endgoal of the project is to have a working programming language that is easy to learn and somewhat efficient... this will take a long time though... 

## Readme info

This project is written and maintained in the scope of a Maturapaper.
**note**
Thorlang is highly experimental and still in development, using it in production is not recommended.


## Installation 

**Note That this section might undergo a lot of change in the future**

### Binaries
Thorlang provides binaries as releases, that can be installed and then executed.

### From source
Thorlang can be installed from source and built using cargo and (on linux) quickly converted to a usable: 

```
git clone https://github.com/Adotweb/thorlang 
cd thorlang 
cargo build --release

#only works on linux
chmod +x ./simple_install.sh 
./simple_install.sh
```


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
|Array| any of the former between [] and seperated by commas |
|Object| variables that have fields of the former types accessed by strings|
|nil| is just `nil`|

To declare an object initialize a variable and put the fields on it. 

### strings, arrays and objects

strings and array can be accessed using brackets like this: 

```thor
    let string = "hello";

    let array = ["a", "b"];

    print string[0]; //prints "h"

    print array[1]; //prints "b"
```

Objects don't have shorthand initialization (yet), instead the fields have to be initialized one by one:

```thor
  let obj;
  obj.field = 0;

  //or use the dynamic field operator
  obj["field"] = 0;

  //to get the field do this
  print obj["field"];

  //or this
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

whenever an operation fails thorlang will try to find overloadings for the given operator and evaluate its logic before failing.

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

**Note** This feauture is still in development and only works when executing a file in the current directory that only imports files from the same directory.

#### Native functions

Of course thorlang has some native functions (this list will be expanded):

| Functions | arguments     | Description                       |
| :-------- | :------- | :-------------------------------- |
| `printf`      | `value : any` | prints `thing` (this is the functional representation of the print statement)|
| `get_input` | `message : string` | prints the message to the terminal and accepts an input (text) which it returns | 
| `getTime` | No arguments | returns the current unix time (unimplemented)| 
| `import` | `filename : any` | returns the returned value of the given file and throws if the file does not exist| 
| `isError` | `any` | returns true if the argument provided is an error and false else| 
| `get_now` | No arguments | returns the current unix time in milliseconds | 
| `cast_to` | `value : any, type : string` | tries to cast the first value to the type thats provided through a string | 

#### Native Methods

And of course Thorlangs native types have methods on them to make your life easier, some of them are listed here (this list will be expanded):

|Type | method | arguments | Description|
|---|---|---|---|
|Number| sqrt | none | returns the square root of the number the method was called on| 
|Array | len | none | returns the length of the array the method was called on |
|Array | push | value | pushes the value to the array and returns the new array|
|String| len | none| returns the length of the string|

## Roadmap 

- [x] turing completeness
- [x] removing all weird bugs/making everything stable
- [x] removing bad code/pretty- and smartifying everything ive written so far
- [x] better errors and error handling
- [ ] easifying installation and documentation
- [ ] std library (for example weblib)
