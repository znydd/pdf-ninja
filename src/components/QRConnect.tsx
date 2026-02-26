import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { QRCodeSVG } from "qrcode.react";

interface ServerInfo {
  running: boolean;
  url: string | null;
}

export function QRConnect() {
  const [serverUrl, setServerUrl] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleStart = async () => {
    setLoading(true);
    setError(null);
    try {
      const url = await invoke<string>("start_upload_server");
      setServerUrl(url);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleStop = async () => {
    setLoading(true);
    setError(null);
    try {
      await invoke("stop_upload_server");
      setServerUrl(null);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <section className="qr-connect">
      {!serverUrl ? (
        <div className="qr-connect__start">
          <button
            className="btn btn-primary"
            onClick={handleStart}
            disabled={loading}
          >
            {loading ? "Starting..." : "📱 Connect Phone"}
          </button>
          {error && <p className="qr-connect__error">{error}</p>}
        </div>
      ) : (
        <div className="qr-connect__active">
          <p className="qr-connect__label">Scan with your phone</p>
          <div className="qr-connect__code">
            <QRCodeSVG
              value={serverUrl}
              size={200}
              bgColor="transparent"
              fgColor="#e8e8f0"
              level="M"
            />
          </div>
          <p className="qr-connect__url">{serverUrl}</p>
          <button
            className="btn btn-danger"
            onClick={handleStop}
            disabled={loading}
          >
            {loading ? "Stopping..." : "Stop Server"}
          </button>
        </div>
      )}
    </section>
  );
}
