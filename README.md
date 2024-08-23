# Thorlang 

## Overview 

I build Thorlang (or thor for short) to understand the steps necessary in order to build a interpreted language from the ground up. The endgoal of the project is to have a working programming language that is easy to learn and somewhat efficient... this will take a long time though... 

## Readme info

This project is written and maintained in the scope of a Maturapaper.


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


#### Modules

Thorlang supports modules, to import a module use the import function:

```thor

let something = import("module.thor");

print something;

```

To export something use the return statement at the end of a thor file: 

```thor

let something = 10;


return something;
```

#### Native functions

Of course thorlang has some native functions (this list will be expanded):

| Functions | arguments     | Description                       |
| :-------- | :------- | :-------------------------------- |
| `printf`      | `thing` | prints `thing` |
| `getTime` | No arguments | returns the current unix time (unimplemented)| 


#### Native Methods

And of course Thorlangs native types have methods on them to make your life easier, some of them are listed here (this list will be expanded):

|Type | method | arguments | Description|
|---|---|---|---|
|Number| sqrt | none | returns the square root of the number the method was called on| 
|Array | len | none | returns the length of the array the method was called on |
|Array | push | value | pushes the value to the array and returns the new array|


