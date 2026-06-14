import { useEffect, useState, useCallback } from "react";
import { Layout } from "./components/Layout";
import { LibraryView } from "./views/LibraryView";
import { AudioView } from "./views/AudioView";
import { initApp } from "./lib/library";
import { useSessionStore } from "./lib/store";
import type { Song } from "./lib/types";
import "./App.css";

function App() {
  const [currentView, setCurrentView] = useState<"library" | "audio">("library");
  const [selectedSong, setSelectedSong] = useState<Song | null>(null);
  const setCurrentSongId = useSessionStore((s) => s.setCurrentSongId);

  useEffect(() => {
    initApp().then(console.log).catch(console.error);
  }, []);

  const handleOpenSong = useCallback(
    (song: Song) => {
      setSelectedSong(song);
      setCurrentSongId(song.id);
      setCurrentView("audio");
    },
    [setCurrentSongId],
  );

  const handleBackToLibrary = useCallback(() => {
    setSelectedSong(null);
    setCurrentView("library");
  }, []);

  return (
    <Layout>
      {currentView === "library" ? (
        <LibraryView onOpenSong={handleOpenSong} />
      ) : (
        selectedSong && (
          <AudioView song={selectedSong} onBack={handleBackToLibrary} />
        )
      )}
    </Layout>
  );
}

export default App;
