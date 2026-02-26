import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <main className="container">
      <h1>PDF Ninja</h1>
      <p>PDF Ninja is a tool for editing PDF files.</p>
      <input type="text" placeholder="Enter your name" value={name} onChange={(e) => setName(e.target.value)} />
      <button onClick={greet}>Greet</button>
      <p>{greetMsg}</p>
    </main>
  );
}

export default App;
