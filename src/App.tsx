import { QRConnect } from "./components/QRConnect";
import { useImageReceiver } from "./hooks/useImageReceiver";
import "./App.css";

function App() {
  const { images, clearImages } = useImageReceiver();

  return (
    <main className="container">
      <h1>PDF Ninja</h1>
      <p className="app-subtitle">Send images from your phone to this PC</p>

      <QRConnect />

      {images.length > 0 && (
        <section className="gallery">
          <div className="gallery__header">
            <h2>Received Images ({images.length})</h2>
            <button className="btn btn-text" onClick={clearImages}>
              Clear
            </button>
          </div>
          <div className="gallery__grid">
            {images.map((url, i) => (
              <div key={i} className="gallery__item">
                <img
                  src={url}
                  alt={`Upload ${i + 1}`}
                  loading="lazy"
                />
              </div>
            ))}
          </div>
        </section>
      )}
    </main>
  );
}

export default App;
