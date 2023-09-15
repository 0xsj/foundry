const person = {
  fullName: {
    firstName: "jayjay",
    lastName: "kim",
  },
  age: 250,
  country: "Finland",
  job: "Instructor and Developer",
  skills: {
    frontend: {
      frameworks: "react",
      library: "something else",
    },
    backend: {
      frameworks: "node",
    },
  },
  languages: ["Amharic", "English", "Suomi(Finnish)"],
};

const {
  skills: { frontend },
} = person;

// Use the filter method to check if 'frameworks' property is 'react'
if (frontend.frameworks === "react") {
  console.log("The person has frontend skills in React.");
} else {
  console.log("The person does not have frontend skills in React.");
}
