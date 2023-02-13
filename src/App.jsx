import { useState } from "react";
import data from "./characters.json";
import "./App.css";

function Character({ name, hp, ac, initiative }) {
  return (
    <div className="character">
      <p>Name: {name}</p>
      <p>HP: {hp}</p>
      <p>AC: {ac}</p>
      <p>Initiative: {initiative}</p>
    </div>
  );
}

function CharacterWrapper({ characters }) {
  const charArray = characters.map((character) => {
    <Character
      name={character["Name"]}
      hp={character["HP"]}
      ac={character["AC"]}
      initiative={character["Initiative"]}
    />
  });

  return (
    <div className="character-wrapper">
      {charArray}
      <button onClick={() => console.log(charArray)}>Debug</button>
    </div>
  );
}

function App() {

  return (
    <>
      <h1>Characters</h1>
      <CharacterWrapper characters={data["Characters"]} />
    </>
  );
}

export default App;
