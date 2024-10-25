// This is typescript, and not valid javascript.
interface Person {
  name: string;
  age: number;
}

function greet(person: Person): string {
  return `Hello, ${person.name}. You are ${person.age} years old.`;
}

const user: Person = {
  name: "Bob",
  age: 42,
};

console.log(greet(user));
