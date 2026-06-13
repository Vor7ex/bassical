import { useEffect } from "react";
import { Layout } from "./components/Layout";
import { LibraryView } from "./views/LibraryView";
import { initApp } from "./lib/library";
import "./App.css";

function App() {
  useEffect(() => {
    initApp().then(console.log).catch(console.error);
  }, []);

  return (
    <Layout>
      <LibraryView />
    </Layout>
  );
}

export default App;
