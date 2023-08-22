When, Where and How to use the arrow function syntax!!
Arrow functions were first introduced in ES6 (ECMAScript 2015). They provide a concise and convenient syntax for defining functions. Let's discuss how,when and where..

The traditional function for adding two numbers can be given as :

const add = function(a, b) {
return a + b;
}
When the above code is converted to arrow function we can skip the return part as an arrow functions automatically return the value of the expression after the arrow (=>) symbol, without the need for an explicit return statement.
const add = (a, b) => a + b;
Lexical this : One of the best thing that arrow function syntax introduced to us

Arrow function has lexical this binding i.e they inherit the this value of the enclosing scope. This can be particularly useful when working with event listeners or callback functions, where the this value can be unpredictable.
For example in the following example the arrow function used as the event listener for the click event inherits the this value of the enclosing scope (which is the global window object), rather than having its own this value like a traditional function would.
const button = document.querySelector('button');
button.addEventListener('click', () => {
console.log(this); // logs the window object, not the button
});
When not to use the Arrow function syntax:

Arrow functions cannot be used as constructors to create new objects, because they do not have their own this value. If you need to create objects using the new keyword, you will need to use a traditional function instead.
Arrow functions cannot be used as generator functions that use the yield keyword to return multiple values over time. If you need to use the yield keyword, you will need to use a traditional function instead.
You must be thinking what is this yield keyword? well, The yield keyword is used in special functions called generator functions, which can return multiple values over time instead of returning just one value like a normal function. When the yield keyword is encountered in the generator function, it pauses the function and returns a value, and the function can be resumed later to return more values.
Basically its like a pause and resume functionality.
